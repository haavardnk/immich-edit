// color-space: linear scene-referred Rgba16Float in → R16Float luma pyramid out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;
use crate::gpu::texture::STORAGE_SAMPLED;
use crate::gpu::texture_pool::TextureKey;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const LUMA_EXTRACT_WGSL: &str = include_str!("../../../assets/shaders/luma_extract.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LumaExtractParams {
    pub offset: [u32; 2],
    pub _pad: [u32; 2],
}

pub struct LumaPyramidPass {
    pub extract_layout: BindGroupLayout,
    pub extract_pipeline: ComputePipeline,
}

impl LumaPyramidPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let extract_layout = make_layout(
            ctx,
            "luma-extract-bgl",
            &[
                uniform_entry(0, size_of::<LumaExtractParams>() as u64),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let extract_pipeline =
            make_pipeline(ctx, &extract_layout, "luma_extract.wgsl", LUMA_EXTRACT_WGSL);
        Self {
            extract_layout,
            extract_pipeline,
        }
    }

    pub fn pyramid_key(ctx: &GpuContext, w: u32, h: u32, levels: u32) -> TextureKey {
        TextureKey::new(ctx.linear_format, w, h, levels, STORAGE_SAMPLED)
    }
}
