use std::borrow::Cow;

use wgpu::util::DeviceExt;

use super::{GpuRoute, LinearImage, Op, OpContext, OpScratch, RenderContext, default_registry};
use crate::edits::{
    BasicEdits, ColorEdits, ColorGradeEdits, ColorGradeRegion, CurvePoint, CurvePoints,
    CurvesEdits, Edits, HslBand, HslEdits, ToneEdits,
};
use crate::gpu::context::GpuContext;

const LEVELS: [f32; 9] = [0.0, 0.015, 0.09, 0.25, 0.5, 0.82, 1.0, 1.7, 3.0];

fn probe_colors() -> Vec<[f32; 3]> {
    LEVELS
        .iter()
        .flat_map(|r| {
            LEVELS
                .iter()
                .flat_map(|g| LEVELS.iter().map(|b| [*r, *g, *b]))
        })
        .collect()
}

fn shadow_luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn probe_ctx() -> OpContext {
    OpContext {
        render: RenderContext {
            wb_coeffs: [1.9, 1.0, 1.55, 1.0],
            cam_to_srgb: [
                [1.42, -0.36, -0.06],
                [-0.15, 1.28, -0.13],
                [0.02, -0.31, 1.29],
            ],
            is_raw: true,
            capture_sigma: None,
            preview_mode: crate::frame::PreviewMode::None,
            roi: None,
            dcp: None,
        },
        scratch: OpScratch::default(),
    }
}

fn probe_edits() -> Edits {
    let bump = CurvePoints {
        points: vec![
            CurvePoint { x: 0.0, y: 0.0 },
            CurvePoint { x: 0.45, y: 0.58 },
            CurvePoint { x: 1.0, y: 1.0 },
        ],
    };
    let mut bands = [HslBand::default(); crate::edits::HSL_BANDS];
    for (i, band) in bands.iter_mut().enumerate() {
        let step = i as f64 * 7.0;
        band.hue = -40.0 + step;
        band.sat = 55.0 - step;
        band.lum = -25.0 + step;
    }
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.65,
            brightness: 32.0,
            contrast: -28.0,
            saturation: 45.0,
            vibrance: -37.0,
            wb_temp: 1800.0,
            wb_tint: -22.0,
            curves: CurvesEdits {
                composite: bump.clone(),
                r: bump.clone(),
                g: bump.clone(),
                b: bump.clone(),
                luma: bump,
            },
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -44.0,
            shadows: 61.0,
            blacks: -18.0,
            whites: 27.0,
        },
        color: ColorEdits {
            hsl: HslEdits { bands },
            color_grade: ColorGradeEdits {
                shadows: ColorGradeRegion {
                    hue: 210.0,
                    sat: 40.0,
                    lum: -12.0,
                },
                midtones: ColorGradeRegion {
                    hue: 45.0,
                    sat: 25.0,
                    lum: 8.0,
                },
                highlights: ColorGradeRegion {
                    hue: 320.0,
                    sat: 30.0,
                    lum: 15.0,
                },
                global: ColorGradeRegion {
                    hue: 90.0,
                    sat: 12.0,
                    lum: -4.0,
                },
                balance: -20.0,
                blend: 65.0,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn run_on_gpu(
    ctx: &GpuContext,
    op: &dyn Op,
    uniform: &[f32],
    colors: &[[f32; 3]],
) -> Result<Vec<[f32; 3]>, String> {
    let gpu_op = op.gpu().ok_or("op has no gpu snippet")?;
    let field_ty = if gpu_op.vec4_count == 1 {
        "vec4<f32>".to_string()
    } else {
        format!("array<vec4<f32>, {}>", gpu_op.vec4_count)
    };
    let source = format!(
        r#"struct Params {{
    {field}: {field_ty},
}};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> src: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> dst: array<vec4<f32>>;

var<private> shadows_blur_l: f32 = 0.0;

{tone}

{prelude}
{functions}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {{
    let i = gid.x;
    if (i >= arrayLength(&src)) {{ return; }}
    var lin = src[i].rgb;
    shadows_blur_l = src[i].w;
{apply}
    dst[i] = vec4<f32>(lin, 1.0);
}}
"#,
        field = gpu_op.field_name,
        field_ty = field_ty,
        tone = crate::tone::wgsl::tone_wgsl(),
        prelude = super::wgsl::op_prelude_wgsl(),
        functions = gpu_op.functions,
        apply = gpu_op.apply,
    );

    let module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("op-parity"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(source)),
        });
    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("op-parity"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    let padded: Vec<f32> = colors
        .iter()
        .flat_map(|c| [c[0], c[1], c[2], shadow_luma(*c)])
        .collect();
    let bytes = colors.len() * 16;
    let src_buf = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("op-parity-src"),
            contents: bytemuck::cast_slice(&padded),
            usage: wgpu::BufferUsages::STORAGE,
        });
    let dst_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("op-parity-dst"),
        size: bytes as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("op-parity-read"),
        size: bytes as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let uni_buf = ctx
        .device
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("op-parity-uniform"),
            contents: bytemuck::cast_slice(uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });

    let bind = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("op-parity"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uni_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: src_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: dst_buf.as_entire_binding(),
            },
        ],
    });

    let mut enc = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.dispatch_workgroups(colors.len().div_ceil(64) as u32, 1, 1);
    }
    enc.copy_buffer_to_buffer(&dst_buf, 0, &read_buf, 0, bytes as u64);
    ctx.queue.submit(Some(enc.finish()));

    crate::gpu::readback::map_buffer_cancellable(ctx, &read_buf, None)
        .map_err(|e| e.to_string())?;
    let slice = read_buf.slice(..);
    let data = crate::gpu::readback::mapped_range(&slice).map_err(|e| e.to_string())?;
    let floats: &[f32] = bytemuck::cast_slice(&data);
    let out = floats.chunks_exact(4).map(|c| [c[0], c[1], c[2]]).collect();
    drop(data);
    read_buf.unmap();
    Ok(out)
}

fn run_on_cpu(op: &dyn Op, edits: &Edits, ctx: &OpContext, colors: &[[f32; 3]]) -> Vec<[f32; 3]> {
    let rgb: Vec<f32> = colors.iter().flat_map(|c| *c).collect();
    let mut image = LinearImage::new(rgb, colors.len(), 1);
    op.apply_cpu(&mut image, ctx, edits).unwrap();
    image
        .rgb
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect()
}

#[test]
fn fused_op_shaders_match_their_rust_implementations() {
    let Ok(gpu) = GpuContext::new() else {
        eprintln!("no gpu adapter, skipping");
        return;
    };
    let edits = probe_edits();
    let colors = probe_colors();
    let mut ctx = probe_ctx();
    ctx.scratch.shadows_blur = Some(std::sync::Arc::new(
        colors.iter().map(|c| shadow_luma(*c)).collect(),
    ));
    let registry = default_registry();
    let mut checked = 0;
    let mut failures: Vec<String> = Vec::new();

    for op in registry.ops() {
        if op.gpu_route() != GpuRoute::Fused {
            continue;
        }
        let Some(gpu_op) = op.gpu() else { continue };
        if gpu_op.vec4_count == 0 {
            continue;
        }
        if !op.is_active(&edits) {
            failures.push(format!("{}: probe edits do not activate the op", op.id()));
            continue;
        }
        let mut uniform = vec![0.0f32; gpu_op.vec4_count * 4];
        op.write_gpu_uniform(&edits, &ctx, &mut uniform);
        let gpu_out = match run_on_gpu(&gpu, op.as_ref(), &uniform, &colors) {
            Ok(v) => v,
            Err(e) => {
                failures.push(format!("{}: {e}", op.id()));
                continue;
            }
        };
        let cpu_out = run_on_cpu(op.as_ref(), &edits, &ctx, &colors);
        let mut worst = 0.0f32;
        let mut worst_at = ([0.0f32; 3], 0.0f32, 0.0f32);
        for (i, (c, g)) in cpu_out.iter().zip(gpu_out.iter()).enumerate() {
            for ch in 0..3 {
                let denom = c[ch].abs().max(g[ch].abs()).max(1e-3);
                let rel = (c[ch] - g[ch]).abs() / denom;
                if rel > worst {
                    worst = rel;
                    worst_at = (colors[i], c[ch], g[ch]);
                }
            }
        }
        eprintln!("SHADER {} worst_rel={worst:.6}", op.id());
        if worst > 1e-4 {
            failures.push(format!(
                "{}: worst_rel={worst:.6} at {:?} cpu={} gpu={}",
                op.id(),
                worst_at.0,
                worst_at.1,
                worst_at.2
            ));
        }
        checked += 1;
    }

    if checked == 0 {
        panic!("no fused ops were checked");
    }
    if !failures.is_empty() {
        panic!("shader/rust divergence:\n{}", failures.join("\n"));
    }
}
