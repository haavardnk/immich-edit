use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferSize,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

pub const DCP_HUESAT_UNIFORM_SIZE: u64 = 1152;

pub struct DcpHueSatPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DcpHueSatPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        Self::with_format(ctx, wgpu::TextureFormat::Rgba16Float, "dcp-huesat")
    }

    pub fn new_look(ctx: &Arc<GpuContext>) -> Self {
        Self::with_format(ctx, wgpu::TextureFormat::Rgba8Unorm, "dcp-look")
    }

    fn with_format(ctx: &Arc<GpuContext>, out_format: wgpu::TextureFormat, label: &str) -> Self {
        let device = &ctx.device;

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some(&format!("{label}-bgl")),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(DCP_HUESAT_UNIFORM_SIZE),
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
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: out_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let src = include_str!("../../../assets/shaders/dcp_huesat.wgsl")
            .replace("rgba16float", storage_format_str(out_format));
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("dcp_huesat.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(src)),
        });
        let pl = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("{label}-pl")),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(&format!("{label}-cp")),
            layout: Some(&pl),
            module: &module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Self { layout, pipeline }
    }
}

fn storage_format_str(f: wgpu::TextureFormat) -> &'static str {
    match f {
        wgpu::TextureFormat::Rgba8Unorm => "rgba8unorm",
        _ => "rgba16float",
    }
}
