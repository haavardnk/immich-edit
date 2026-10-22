// color-space: camera linear Rgba16Float in/out, camera linear denoise plane
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry_unsized};

pub const DENOISE_BLEND_WGSL: &str = include_str!("../../../assets/shaders/denoise_blend.wgsl");

pub struct DenoiseBlendPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DenoiseBlendPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "denoise-blend-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry(1),
                tex_entry(2),
                storage_entry(3, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(ctx, &layout, "denoise_blend.wgsl", DENOISE_BLEND_WGSL);
        Self { layout, pipeline }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DenoiseBlendParams {
    pub size: [u32; 2],
    pub strength: f32,
    pub detail: f32,
    pub luma: [f32; 3],
    pub _pad0: f32,
    pub neutral: [f32; 3],
    pub _pad1: f32,
}
