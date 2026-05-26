use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferSize,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::demosaic::linear_format_str;

pub const SHARPEN_BLUR_UNIFORM_SIZE: u64 = 32;
pub const SHARPEN_COMBINE_UNIFORM_SIZE: u64 = 80;

pub struct OutputSharpenPass {
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: ComputePipeline,
    pub combine_layout: BindGroupLayout,
    pub combine_pipeline: ComputePipeline,
}

impl OutputSharpenPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;

        let blur_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sharpen-blur-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(SHARPEN_BLUR_UNIFORM_SIZE),
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
                        format: ctx.linear_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let blur_src = include_str!("../../../assets/shaders/sharpen_blur.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format));
        let blur_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("sharpen_blur.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(blur_src)),
        });
        let blur_pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sharpen-blur-pl"),
            bind_group_layouts: &[&blur_layout],
            push_constant_ranges: &[],
        });
        let blur_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("sharpen-blur-cp"),
            layout: Some(&blur_pl),
            module: &blur_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        let combine_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sharpen-combine-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(SHARPEN_COMBINE_UNIFORM_SIZE),
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
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 4,
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
        let combine_src = include_str!("../../../assets/shaders/sharpen_combine.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format))
            .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::TONE_WGSL);
        let combine_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("sharpen_combine.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(combine_src)),
        });
        let combine_pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sharpen-combine-pl"),
            bind_group_layouts: &[&combine_layout],
            push_constant_ranges: &[],
        });
        let combine_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("sharpen-combine-cp"),
            layout: Some(&combine_pl),
            module: &combine_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            blur_layout,
            blur_pipeline,
            combine_layout,
            combine_pipeline,
        }
    }
}
