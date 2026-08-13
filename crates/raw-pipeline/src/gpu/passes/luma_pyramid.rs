// color-space: linear scene-referred Rgba16Float in → R16Float luma pyramid out
use std::sync::Arc;

use wgpu::{
    BindGroupLayout, ComputePipeline, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages,
};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry};

pub struct LumaPyramidPass {
    pub extract_layout: BindGroupLayout,
    pub extract_pipeline: ComputePipeline,
}

impl LumaPyramidPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let extract_layout = make_layout(
            ctx,
            "luma-extract-bgl",
            &[tex_entry(0), storage_entry(1, ctx.linear_format)],
        );
        let extract_pipeline = make_pipeline(
            ctx,
            &extract_layout,
            "luma_extract.wgsl",
            include_str!("../../../assets/shaders/luma_extract.wgsl"),
        );
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
