use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    BufferBindingType, ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor,
    SamplerBindingType, ShaderModuleDescriptor, ShaderSource, ShaderStages, StorageTextureAccess,
    TextureFormat, TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;
use crate::gpu::shader_builder::{self, BuiltProcessShader};
use crate::ops::OpRegistry;

pub struct ProcessFastPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
    pub built: BuiltProcessShader,
}

impl ProcessFastPass {
    pub fn new(ctx: &Arc<GpuContext>, registry: &OpRegistry) -> Self {
        let device = &ctx.device;
        let built = shader_builder::build(registry);

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            label: Some("process.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(built.wgsl.clone())),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("process-pl"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("process-cp"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            layout,
            pipeline,
            built,
        }
    }
}
