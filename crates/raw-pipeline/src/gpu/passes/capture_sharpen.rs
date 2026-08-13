// color-space: linear scene-referred Rgba16Float in/out, R32Float luma scratch
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_entry, tex_entry, tex_entry_with, uniform_entry,
};

pub const CAPTURE_LUMA_UNIFORM_SIZE: u64 = size_of::<CaptureLumaParams>() as u64;
pub const CAPTURE_BLUR_UNIFORM_SIZE: u64 = size_of::<CaptureBlurParams>() as u64;
pub const CAPTURE_APPLY_UNIFORM_SIZE: u64 = size_of::<CaptureApplyParams>() as u64;

pub const CAPTURE_KERNEL_MAX: usize = 16;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CaptureLumaParams {
    pub size: [u32; 2],
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CaptureBlurParams {
    pub size: [u32; 2],
    pub radius: u32,
    pub axis: u32,
    pub mode: u32,
    pub _pad: [u32; 3],
    pub kernel: [f32; CAPTURE_KERNEL_MAX],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CaptureApplyParams {
    pub size: [u32; 2],
    pub radius: u32,
    pub _pad: u32,
}
pub const CAPTURE_MAX_TAPS: usize = 16;
pub const CAPTURE_SCRATCH_FORMAT: TextureFormat = TextureFormat::R32Float;

pub struct CaptureSharpenPasses {
    pub luma_layout: BindGroupLayout,
    pub luma_pipeline: ComputePipeline,
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: ComputePipeline,
    pub apply_layout: BindGroupLayout,
    pub apply_pipeline: ComputePipeline,
}

impl CaptureSharpenPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let scratch_tex = |binding: u32| tex_entry_with(binding, false, TextureViewDimension::D2);
        let luma_layout = make_layout(
            ctx,
            "capture-luma-bgl",
            &[
                uniform_entry(0, CAPTURE_LUMA_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, CAPTURE_SCRATCH_FORMAT),
                storage_entry(3, CAPTURE_SCRATCH_FORMAT),
            ],
        );
        let luma_pipeline = make_pipeline(
            ctx,
            &luma_layout,
            "capture_luma.wgsl",
            include_str!("../../../assets/shaders/capture_luma.wgsl"),
        );
        let blur_layout = make_layout(
            ctx,
            "capture-blur-bgl",
            &[
                uniform_entry(0, CAPTURE_BLUR_UNIFORM_SIZE),
                scratch_tex(1),
                scratch_tex(2),
                storage_entry(3, CAPTURE_SCRATCH_FORMAT),
            ],
        );
        let blur_pipeline = make_pipeline(
            ctx,
            &blur_layout,
            "capture_blur.wgsl",
            include_str!("../../../assets/shaders/capture_blur.wgsl"),
        );
        let apply_layout = make_layout(
            ctx,
            "capture-apply-bgl",
            &[
                uniform_entry(0, CAPTURE_APPLY_UNIFORM_SIZE),
                tex_entry(1),
                scratch_tex(2),
                scratch_tex(3),
                storage_entry(4, ctx.linear_format),
            ],
        );
        let apply_pipeline = make_pipeline(
            ctx,
            &apply_layout,
            "capture_apply.wgsl",
            include_str!("../../../assets/shaders/capture_apply.wgsl"),
        );
        Self {
            luma_layout,
            luma_pipeline,
            blur_layout,
            blur_pipeline,
            apply_layout,
            apply_pipeline,
        }
    }
}
