// color-space: linear scene-referred Rgba16Float in/out
use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferSize,
    ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StorageTextureAccess, TextureSampleType, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::demosaic::linear_format_str;

pub const PRESENCE_UNIFORM_SIZE: u64 = 48;

pub struct PresencePass {
    pub adjust_layout: BindGroupLayout,
    pub adjust_pipeline: ComputePipeline,
}

impl PresencePass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let adjust_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("presence-adjust-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(PRESENCE_UNIFORM_SIZE),
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
                        format: ctx.linear_format,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let src = include_str!("../../../assets/shaders/presence_adjust.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format));
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("presence_adjust.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(src)),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("presence-adjust-pl"),
            bind_group_layouts: &[&adjust_layout],
            push_constant_ranges: &[],
        });
        let adjust_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("presence-adjust-cp"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            adjust_layout,
            adjust_pipeline,
        }
    }
}

pub fn select_mip(max_edge: u32, radius_px: u32) -> u32 {
    if radius_px <= 1 {
        return 0;
    }
    let target = (radius_px as f32).log2().round() as i32;
    let max_levels = (max_edge as f32).log2().floor() as i32 + 1;
    target.clamp(0, max_levels - 1) as u32
}
