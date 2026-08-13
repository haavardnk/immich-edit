// color-space: linear scene-referred Rgba16Float in/out; tone-map applied later in effects_tone.wgsl
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_entry, tex_entry, tex_entry_with, uniform_entry,
};

pub const SHARPEN_BLUR_UNIFORM_SIZE: u64 = size_of::<SharpenBlurParams>() as u64;
pub const SHARPEN_UNIFORM_SIZE: u64 = size_of::<SharpenParams>() as u64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SharpenBlurParams {
    pub sigma: f32,
    pub radius: f32,
    pub size: [u32; 2],
    pub axis: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SharpenParams {
    pub amount: f32,
    pub detail_weight: f32,
    pub masking_thresh: f32,
    pub masking_softness: f32,
    pub size: [u32; 2],
    pub use_mask: u32,
    pub preview_mode: u32,
    pub masked_sharpen: u32,
    pub _pad: [u32; 3],
}

pub struct OutputSharpenPass {
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: ComputePipeline,
    pub sharpen_layout: BindGroupLayout,
    pub sharpen_pipeline: ComputePipeline,
}

impl OutputSharpenPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let blur_layout = make_layout(
            ctx,
            "sharpen-blur-bgl",
            &[
                uniform_entry(0, SHARPEN_BLUR_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let blur_pipeline = make_pipeline(
            ctx,
            &blur_layout,
            "sharpen_blur.wgsl",
            include_str!("../../../assets/shaders/sharpen_blur.wgsl"),
        );

        let sharpen_layout = make_layout(
            ctx,
            "sharpen-bgl",
            &[
                uniform_entry(0, SHARPEN_UNIFORM_SIZE),
                tex_entry(1),
                tex_entry(2),
                storage_entry(4, ctx.linear_format),
                tex_entry_with(5, false, TextureViewDimension::D2),
            ],
        );
        let sharpen_pipeline = make_pipeline(
            ctx,
            &sharpen_layout,
            "sharpen.wgsl",
            include_str!("../../../assets/shaders/sharpen.wgsl"),
        );

        Self {
            blur_layout,
            blur_pipeline,
            sharpen_layout,
            sharpen_pipeline,
        }
    }
}
