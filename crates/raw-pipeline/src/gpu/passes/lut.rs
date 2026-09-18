use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureViewDimension};

use crate::gpu::context::GpuContext;
use crate::gpu::display_depth::{DISPLAY_LOAD_INJECT, DISPLAY_STORE_INJECT, DisplayDepth};

use super::common::{
    display_src_entry, make_layout, make_pipeline, storage_entry, tex_entry_with, uniform_entry,
};

pub const LUT_UNIFORM_SIZE: u64 = size_of::<LutParams>() as u64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LutParams {
    pub size: [u32; 3],
    pub _pad0: u32,
    pub domain_min: [f32; 3],
    pub _pad1: f32,
    pub domain_max: [f32; 3],
    pub _pad2: f32,
    pub amount: f32,
    pub _pad3: [f32; 3],
}

pub struct LutPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl LutPass {
    pub fn new(ctx: &Arc<GpuContext>, depth: DisplayDepth) -> Self {
        let layout = make_layout(
            ctx,
            "lut-bgl",
            &[
                uniform_entry(0, LUT_UNIFORM_SIZE),
                display_src_entry(1, depth),
                tex_entry_with(2, false, TextureViewDimension::D3),
                storage_entry(3, depth.format()),
            ],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "lut.wgsl",
            &include_str!("../../../assets/shaders/lut.wgsl")
                .replace(DISPLAY_LOAD_INJECT, &depth.load_wgsl(1))
                .replace(DISPLAY_STORE_INJECT, &depth.store_wgsl(3))
                .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl()),
        );

        Self { layout, pipeline }
    }
}
