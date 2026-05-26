use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use wgpu::{
    AddressMode, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ComputePipeline, ComputePipelineDescriptor, FilterMode,
    PipelineLayoutDescriptor, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;
use crate::mask_raster::MaskRaster;

pub const COMPONENT_BYTES: usize = 64;
pub const MAX_COMPONENTS: usize = 32;
pub const MAX_COMPONENTS_BYTES: usize = COMPONENT_BYTES * MAX_COMPONENTS;
pub const PARAMS_BYTES: usize = 96;
pub const ATLAS_DIM: u32 = 1024;
pub const ATLAS_LAYERS: u32 = 16;

const SHADER: &str = r#"
struct MaskParams {
    out_size: vec2<u32>,
    n_components: u32,
    layer_amount: f32,
    crop: vec4<f32>,
    flags: vec4<u32>,
    geom_extra2: vec4<f32>,
    geom_extra3: vec4<f32>,
    lens: vec4<f32>,
};

struct Component {
    kind_mode_invert_pad: vec4<u32>,
    opacity_pad: vec4<f32>,
    geom_a: vec4<f32>,
    geom_b: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: MaskParams;
@group(0) @binding(1) var<storage, read> comps: array<Component>;
@group(0) @binding(2) var weight_out: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var atlas: texture_2d_array<f32>;
@group(0) @binding(4) var samp: sampler;

fn smoothstep_calc(e0: f32, e1: f32, x: f32) -> f32 {
    let t = clamp((x - e0) / max(e1 - e0, 1e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn display_to_scene(disp_u: f32, disp_v: f32) -> vec2<f32> {
    let bx_rel = p.crop.x + disp_u * p.crop.z;
    let by_rel = p.crop.y + disp_v * p.crop.w;
    let cx_px = (bx_rel - 0.5) * p.geom_extra2.z;
    let cy_px = (by_rel - 0.5) * p.geom_extra2.w;
    let sx_px = cx_px * p.geom_extra2.x + cy_px * p.geom_extra2.y;
    let sy_px = -cx_px * p.geom_extra2.y + cy_px * p.geom_extra2.x;
    let u = sx_px / p.geom_extra3.x + 0.5;
    let v = sy_px / p.geom_extra3.y + 0.5;
    let rot = p.flags.x;
    let flip_h = p.flags.y;
    let flip_v = p.flags.z;
    var cu = u;
    var cv = v;
    if (flip_h == 1u) { cu = 1.0 - cu; }
    if (flip_v == 1u) { cv = 1.0 - cv; }
    var mu: f32;
    var mv: f32;
    if (rot == 90u) { mu = cv; mv = 1.0 - cu; }
    else if (rot == 180u) { mu = 1.0 - cu; mv = 1.0 - cv; }
    else if (rot == 270u) { mu = 1.0 - cv; mv = cu; }
    else { mu = cu; mv = cv; }
    let k1 = p.lens.x;
    let k2 = p.lens.y;
    let k3 = p.lens.z;
    let zoom = p.lens.w;
    if (k1 == 0.0 && k2 == 0.0 && k3 == 0.0 && zoom == 1.0) {
        return vec2<f32>(mu, mv);
    }
    let mw = p.geom_extra3.z;
    let mh = p.geom_extra3.w;
    let half_diag = 0.5 * sqrt(mw * mw + mh * mh);
    let nx = (mu - 0.5) * mw;
    let ny = (mv - 0.5) * mh;
    let r = sqrt(nx * nx + ny * ny) * zoom / max(half_diag, 1e-6);
    let r2 = r * r;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
    return vec2<f32>(0.5 + (mu - 0.5) * zoom * s, 0.5 + (mv - 0.5) * zoom * s);
}

fn component_weight(c: Component, u: f32, v: f32) -> f32 {
    var raw: f32 = 0.0;
    let kind = c.kind_mode_invert_pad.x;
    if (kind == 0u) {
        let p0x = c.geom_a.x;
        let p0y = c.geom_a.y;
        let dx = c.geom_a.z;
        let dy = c.geom_a.w;
        let len2 = max(c.geom_b.x, 1e-12);
        let feather = clamp(c.geom_b.y, 0.0, 1.0);
        let t = ((u - p0x) * dx + (v - p0y) * dy) / len2;
        let half_f = 0.5 * feather;
        raw = smoothstep_calc(0.5 - half_f, 0.5 + half_f, t);
    } else if (kind == 1u) {
        let cx = c.geom_a.x;
        let cy = c.geom_a.y;
        let inv_rx = c.geom_a.z;
        let inv_ry = c.geom_a.w;
        let feather = clamp(c.geom_b.y, 0.0, 1.0);
        let ddx = (u - cx) * inv_rx;
        let ddy = (v - cy) * inv_ry;
        let d = sqrt(ddx * ddx + ddy * ddy);
        raw = 1.0 - smoothstep_calc(1.0 - max(feather, 1e-3), 1.0, d);
    } else if (kind == 2u) {
        let slot = i32(c.kind_mode_invert_pad.w);
        raw = textureSampleLevel(atlas, samp, vec2<f32>(u, v), slot, 0.0).x;
    }
    let inverted = c.kind_mode_invert_pad.z;
    var r = raw;
    if (inverted == 1u) { r = 1.0 - r; }
    let op = clamp(c.opacity_pad.x, 0.0, 1.0);
    return clamp(r * op, 0.0, 1.0);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let ow = f32(p.out_size.x);
    let oh = f32(p.out_size.y);
    let du = (f32(gid.x) + 0.5) / ow;
    let dv = (f32(gid.y) + 0.5) / oh;
    let scene = display_to_scene(du, dv);
    let u = scene.x;
    let v = scene.y;
    var w: f32 = 0.0;
    let n = p.n_components;
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let c = comps[i];
        let cw = component_weight(c, u, v);
        let mode = c.kind_mode_invert_pad.y;
        if (mode == 0u) {
            w = 1.0 - (1.0 - w) * (1.0 - cw);
        } else if (mode == 1u) {
            w = w * (1.0 - cw);
        } else {
            w = w * cw;
        }
    }
    let final_w = clamp(w * p.layer_amount, 0.0, 1.0);
    textureStore(weight_out, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(final_w, 0.0, 0.0, 1.0));
}
"#;

pub struct MaskWeightPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskWeightPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mask-weight-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::R32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("mask-weight.wgsl"),
            source: ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mask-weight-pl"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("mask-weight-cp"),
            layout: Some(&pl),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Self { layout, pipeline }
    }
}

pub fn make_atlas_sampler(ctx: &Arc<GpuContext>) -> Sampler {
    ctx.device.create_sampler(&SamplerDescriptor {
        label: Some("mask-atlas-sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Nearest,
        ..Default::default()
    })
}

pub fn resample_raster_to_atlas(raster: &MaskRaster) -> Vec<u8> {
    let dim = ATLAS_DIM as usize;
    let mut out = vec![0u8; dim * dim];
    let inv = 1.0 / dim as f32;
    for y in 0..dim {
        let v = (y as f32 + 0.5) * inv;
        let row = y * dim;
        for x in 0..dim {
            let u = (x as f32 + 0.5) * inv;
            let w = raster.sample_bilinear(u, v).clamp(0.0, 1.0);
            out[row + x] = (w * 255.0 + 0.5) as u8;
        }
    }
    out
}

pub fn pack_layer_eval(
    layer: &crate::cpu::masked::LayerEval,
    slot_map: &HashMap<String, u32>,
) -> (Vec<u8>, u32) {
    let mut out = Vec::with_capacity(layer.components.len() * COMPONENT_BYTES);
    let mut n: u32 = 0;
    for c in &layer.components {
        if n as usize >= MAX_COMPONENTS {
            break;
        }
        let mut slot: u32 = 0;
        let (kind, geom_a, geom_b) = match &c.kind {
            crate::cpu::masked::ComponentKindEval::Linear {
                p0,
                dir,
                len2,
                feather,
            } => (
                0u32,
                [p0.0, p0.1, dir.0, dir.1],
                [*len2, *feather, 0.0, 0.0],
            ),
            crate::cpu::masked::ComponentKindEval::Radial {
                center,
                inv_radius,
                feather,
            } => (
                1u32,
                [center.0, center.1, inv_radius.0, inv_radius.1],
                [0.0, *feather, 0.0, 0.0],
            ),
            crate::cpu::masked::ComponentKindEval::Brush { raster_id, raster } => {
                if raster.is_none() {
                    continue;
                }
                let Some(s) = slot_map.get(raster_id) else {
                    continue;
                };
                slot = *s;
                (2u32, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0])
            }
        };
        let mode = match c.mode {
            crate::edits::MaskComponentMode::Add => 0u32,
            crate::edits::MaskComponentMode::Subtract => 1u32,
            crate::edits::MaskComponentMode::Intersect => 2u32,
        };
        let invert = if c.invert { 1u32 } else { 0u32 };
        let opacity = c.opacity.clamp(0.0, 1.0);
        let mut buf = [0u8; COMPONENT_BYTES];
        buf[0..4].copy_from_slice(&kind.to_ne_bytes());
        buf[4..8].copy_from_slice(&mode.to_ne_bytes());
        buf[8..12].copy_from_slice(&invert.to_ne_bytes());
        buf[12..16].copy_from_slice(&slot.to_ne_bytes());
        buf[16..20].copy_from_slice(&opacity.to_ne_bytes());
        for (i, f) in geom_a.iter().enumerate() {
            buf[32 + i * 4..36 + i * 4].copy_from_slice(&f.to_ne_bytes());
        }
        for (i, f) in geom_b.iter().enumerate() {
            buf[48 + i * 4..52 + i * 4].copy_from_slice(&f.to_ne_bytes());
        }
        out.extend_from_slice(&buf);
        n += 1;
    }
    (out, n)
}

#[allow(clippy::too_many_arguments)]
pub fn pack_params(
    out_w: u32,
    out_h: u32,
    n_components: u32,
    layer_amount: f32,
    crop: [f32; 4],
    flags: [u32; 4],
    geom_extra2: [f32; 4],
    geom_extra3: [f32; 4],
    lens: [f32; 4],
) -> [u8; PARAMS_BYTES] {
    let mut buf = [0u8; PARAMS_BYTES];
    buf[0..4].copy_from_slice(&out_w.to_ne_bytes());
    buf[4..8].copy_from_slice(&out_h.to_ne_bytes());
    buf[8..12].copy_from_slice(&n_components.to_ne_bytes());
    buf[12..16].copy_from_slice(&layer_amount.to_ne_bytes());
    buf[16..32].copy_from_slice(bytemuck::cast_slice(&crop));
    buf[32..48].copy_from_slice(bytemuck::cast_slice(&flags));
    buf[48..64].copy_from_slice(bytemuck::cast_slice(&geom_extra2));
    buf[64..80].copy_from_slice(bytemuck::cast_slice(&geom_extra3));
    buf[80..96].copy_from_slice(bytemuck::cast_slice(&lens));
    buf
}
