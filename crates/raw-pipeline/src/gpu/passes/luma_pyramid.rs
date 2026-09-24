// color-space: linear scene-referred Rgba16Float in → R16Float luma pyramid out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureUsages};

use crate::gpu::context::GpuContext;
use crate::gpu::texture_pool::TextureKey;

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

    pub fn pyramid_key(ctx: &GpuContext, w: u32, h: u32, levels: u32) -> TextureKey {
        TextureKey::new(
            ctx.linear_format,
            w,
            h,
            levels,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        )
    }
}
