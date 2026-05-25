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
struct BlendParams {
    out_size: vec2<u32>,
    pad: vec2<u32>,
};

@group(0) @binding(0) var<uniform> p: BlendParams;
@group(0) @binding(1) var curr_tex: texture_2d<f32>;
@group(0) @binding(2) var layer_tex: texture_2d<f32>;
@group(0) @binding(3) var weight_tex: texture_2d<f32>;
@group(0) @binding(4) var dst_tex: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(curr_tex, coord, 0).rgb;
    let l = textureLoad(layer_tex, coord, 0).rgb;
    let w = clamp(textureLoad(weight_tex, coord, 0).r, 0.0, 1.0);
    let outc = c + (l - c) * w;
    textureStore(dst_tex, coord, vec4<f32>(outc, 1.0));
}
"#;

pub struct MaskBlendPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskBlendPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mask-blend-bgl"),
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
                tex_entry_r(3),
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("mask-blend.wgsl"),
            source: ShaderSource::Wgsl(Cow::Borrowed(SHADER)),
        });
        let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mask-blend-pl"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("mask-blend-cp"),
            layout: Some(&pl),
            module: &module,
            entry_point: "main",
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

fn tex_entry_r(binding: u32) -> BindGroupLayoutEntry {
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

pub fn pack_params(out_w: u32, out_h: u32) -> [u8; PARAMS_BYTES] {
    let mut buf = [0u8; PARAMS_BYTES];
    buf[0..4].copy_from_slice(&out_w.to_ne_bytes());
    buf[4..8].copy_from_slice(&out_h.to_ne_bytes());
    buf
}
