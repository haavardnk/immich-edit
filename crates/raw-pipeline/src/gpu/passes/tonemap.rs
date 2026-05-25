use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

pub const PARAMS_BYTES: usize = 16;

const SHADER: &str = r#"
struct TonemapParams {
    out_size: vec2<u32>,
    pad: vec2<u32>,
};

@group(0) @binding(0) var<uniform> p: TonemapParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var dst_tex: texture_storage_2d<rgba8unorm, write>;

fn soft_clip_high(v: f32) -> f32 {
    let knee: f32 = 0.95;
    if (v <= knee) { return v; }
    let headroom: f32 = 1.0 - knee;
    let excess: f32 = v - knee;
    return knee + headroom * (excess / (excess + headroom));
}

fn default_tone(v: f32) -> f32 {
    var lin: f32;
    if (v <= 0.0) { lin = 0.0; } else { lin = soft_clip_high(v); }
    var srgb: f32;
    if (lin <= 0.003130808) {
        srgb = 12.92 * lin;
    } else {
        srgb = 1.055 * pow(lin, 1.0 / 2.4) - 0.055;
    }
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    return srgb + (s - srgb) * 0.15;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(src_tex, coord, 0).rgb;
    let outc = vec3<f32>(default_tone(c.r), default_tone(c.g), default_tone(c.b));
    textureStore(dst_tex, coord, vec4<f32>(outc, 1.0));
}
"#;

pub struct TonemapPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl TonemapPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("tonemap-bgl"),
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
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("tonemap.wgsl"),
            source: ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("tonemap-pl"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("tonemap-cp"),
            layout: Some(&pl),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Self { layout, pipeline }
    }
}

pub fn pack_params(out_w: u32, out_h: u32) -> [u8; PARAMS_BYTES] {
    let mut buf = [0u8; PARAMS_BYTES];
    buf[0..4].copy_from_slice(&out_w.to_ne_bytes());
    buf[4..8].copy_from_slice(&out_h.to_ne_bytes());
    buf
}
