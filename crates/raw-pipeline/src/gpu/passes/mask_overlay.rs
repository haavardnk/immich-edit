// color-space: display-referred Rgba8Unorm in/out (tints the finished image by mask weight)
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
pub const OVERLAY_ALPHA: f32 = 0.55;

const SHADER: &str = r#"
struct OverlayParams {
    out_size: vec2<u32>,
    strength: f32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: OverlayParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var weight_tex: texture_2d<f32>;
@group(0) @binding(3) var dst_tex: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let src = textureLoad(src_tex, coord, 0);
    let w = clamp(textureLoad(weight_tex, coord, 0).r, 0.0, 1.0);
    let a = w * p.strength;
    let outc = vec3<f32>(
        src.r + (1.0 - src.r) * a,
        src.g * (1.0 - a),
        src.b * (1.0 - a),
    );
    textureStore(dst_tex, coord, vec4<f32>(outc, src.a));
}
"#;

pub struct MaskOverlayPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskOverlayPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mask-overlay-bgl"),
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
                tex_entry(1),
                tex_entry(2),
                BindGroupLayoutEntry {
                    binding: 3,
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
            label: Some("mask-overlay.wgsl"),
            source: ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mask-overlay-pl"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("mask-overlay-cp"),
            layout: Some(&pl),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        Self { layout, pipeline }
    }
}

fn tex_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Texture {
            sample_type: TextureSampleType::Float { filterable: false },
            view_dimension: TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

pub fn pack_params(out_w: u32, out_h: u32, strength: f32) -> [u8; PARAMS_BYTES] {
    let mut buf = [0u8; PARAMS_BYTES];
    buf[0..4].copy_from_slice(&out_w.to_ne_bytes());
    buf[4..8].copy_from_slice(&out_h.to_ne_bytes());
    buf[8..12].copy_from_slice(&strength.to_ne_bytes());
    buf
}
