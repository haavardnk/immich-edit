use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferSize,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::demosaic::linear_format_str;

pub const EFFECTS_TONE_UNIFORM_SIZE: u64 = 64;

pub struct EffectsTonePass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl EffectsTonePass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("effects-tone-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(EFFECTS_TONE_UNIFORM_SIZE),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
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
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: ctx.linear_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let src = include_str!("../../../assets/shaders/effects_tone.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format))
            .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl());
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("effects_tone.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(src)),
        });
        let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("effects-tone-pl"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("effects-tone-cp"),
            layout: Some(&pl),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Self { layout, pipeline }
    }
}
