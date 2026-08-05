use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry_with, uniform_entry};

pub const LUT_UNIFORM_SIZE: u64 = 64;

pub struct LutPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl LutPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "lut-bgl",
            &[
                uniform_entry(0, LUT_UNIFORM_SIZE),
                tex_entry_with(1, false, TextureViewDimension::D2),
                tex_entry_with(2, false, TextureViewDimension::D3),
                storage_entry(3, wgpu::TextureFormat::Rgba8Unorm),
            ],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "lut.wgsl",
            &include_str!("../../../assets/shaders/lut.wgsl")
                .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl()),
        );

        Self { layout, pipeline }
    }
}
