// color-space: linear scene-referred Rgba16Float in → display sRGB-encoded Rgba32Float selector out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline_raw, storage_entry, tex_entry_with, uniform_entry_unsized,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaskSelectorParams {
    pub size: [u32; 2],
    pub src_size: [u32; 2],
    pub offset: u32,
    pub mode: u32,
    pub whole: u32,
    pub frac: f32,
    pub eps: f32,
    pub _pad: [u32; 3],
}

pub fn mask_selector_wgsl() -> String {
    include_str!("../../../assets/shaders/mask_selector.wgsl")
        .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl())
}

pub struct MaskSelectorPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskSelectorPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "mask-selector-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry_with(1, false, TextureViewDimension::D2),
                tex_entry_with(2, false, TextureViewDimension::D2),
                tex_entry_with(3, false, TextureViewDimension::D2),
                storage_entry(4, TextureFormat::Rgba16Float),
                storage_entry(5, TextureFormat::Rgba16Float),
                storage_entry(6, TextureFormat::Rgba32Float),
            ],
        );
        let pipeline = make_pipeline_raw(ctx, &layout, "mask-selector-cp", &mask_selector_wgsl());
        Self { layout, pipeline }
    }
}
