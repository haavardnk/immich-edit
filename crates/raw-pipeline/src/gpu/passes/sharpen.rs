// color-space: linear scene-referred Rgba16Float in/out; tone-map applied later in effects_tone.wgsl
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
pub const SHARPEN_UNIFORM_SIZE: u64 = 32;

pub struct OutputSharpenPass {
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: ComputePipeline,
    pub sharpen_layout: BindGroupLayout,
    pub sharpen_pipeline: ComputePipeline,
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

        let sharpen_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sharpen-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(SHARPEN_UNIFORM_SIZE),
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
        let sharpen_src = include_str!("../../../assets/shaders/sharpen.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format));
        let sharpen_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("sharpen.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(sharpen_src)),
        });
        let sharpen_pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("sharpen-pl"),
            bind_group_layouts: &[&sharpen_layout],
            push_constant_ranges: &[],
        });
        let sharpen_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("sharpen-cp"),
            layout: Some(&sharpen_pl),
            module: &sharpen_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            blur_layout,
            blur_pipeline,
            sharpen_layout,
            sharpen_pipeline,
        }
    }
}
