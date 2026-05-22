use std::borrow::Cow;
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, ComputePipeline,
    ComputePipelineDescriptor, Extent3d, PipelineLayoutDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StorageTextureAccess, Texture, TextureDescriptor, TextureDimension,
    TextureSampleType, TextureUsages, TextureViewDimension,
};

use crate::gpu::context::GpuContext;

use super::demosaic::linear_format_str;

pub struct LumaPyramidPass {
    pub extract_layout: BindGroupLayout,
    pub extract_pipeline: ComputePipeline,
}

impl LumaPyramidPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let device = &ctx.device;
        let extract_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("luma-extract-bgl"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
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
        let src = include_str!("../../../assets/shaders/luma_extract.wgsl")
            .replace("rgba16float", linear_format_str(ctx.linear_format));
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("luma_extract.wgsl"),
            source: ShaderSource::Wgsl(Cow::Owned(src)),
        });
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("luma-extract-pl"),
            bind_group_layouts: &[&extract_layout],
            push_constant_ranges: &[],
        });
        let extract_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("luma-extract-cp"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });
        Self {
            extract_layout,
            extract_pipeline,
        }
    }

    pub fn allocate_pyramid(ctx: &GpuContext, w: u32, h: u32, levels: u32) -> Texture {
        ctx.device.create_texture(&TextureDescriptor {
            label: Some("luma-pyramid"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: levels,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}

pub fn pyramid_levels_for(w: u32, h: u32, max_radius_px: u32) -> u32 {
    let max_edge = w.max(h);
    let by_size = (max_edge as f32).log2().floor() as u32 + 1;
    let needed = ((max_radius_px.max(1) as f32).log2().ceil() as u32) + 1;
    needed.min(by_size).max(1)
}
