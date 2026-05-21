use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, SamplerBindingType,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

use super::context::GpuContext;

pub struct GpuPipelines {
    pub demosaic_layout: BindGroupLayout,
    pub demosaic_pipeline: ComputePipeline,
    pub process_layout: BindGroupLayout,
    pub process_pipeline: ComputePipeline,
}

impl GpuPipelines {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;

        let demosaic_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("demosaic-bgl"),
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
                        format: ctx.linear_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let demosaic_src = include_str!("../../assets/shaders/demosaic.wgsl");
        let demosaic_src = demosaic_src
            .replace("rgba16float", linear_format_str(ctx.linear_format));
        let demosaic_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("demosaic.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(demosaic_src)),
        });
        let demosaic_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("demosaic-pl"),
            bind_group_layouts: &[&demosaic_layout],
            push_constant_ranges: &[],
        });
        let demosaic_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("demosaic-cp"),
            layout: Some(&demosaic_pipeline_layout),
            module: &demosaic_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        let process_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("process-bgl"),
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
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
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

        let process_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("process.wgsl"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "../../assets/shaders/process.wgsl"
            ))),
        });
        let process_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("process-pl"),
            bind_group_layouts: &[&process_layout],
            push_constant_ranges: &[],
        });
        let process_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("process-cp"),
            layout: Some(&process_pipeline_layout),
            module: &process_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            demosaic_layout,
            demosaic_pipeline,
            process_layout,
            process_pipeline,
        }
    }
}

fn linear_format_str(fmt: TextureFormat) -> &'static str {
    match fmt {
        TextureFormat::Rgba16Float => "rgba16float",
        TextureFormat::Rgba32Float => "rgba32float",
        _ => "rgba16float",
    }
}
