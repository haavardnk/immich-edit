// color-space: linear scene-referred Rgba16Float in/out, Rgba32Float log-luma guide scratch
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_entry, tex_entry, tex_entry_with, uniform_entry,
};

pub const PRESENCE_UNIFORM_SIZE: u64 = size_of::<PresenceParams>() as u64;
pub const CLARITY_GUIDE_UNIFORM_SIZE: u64 = size_of::<ClarityGuideParams>() as u64;
pub const CLARITY_GUIDE_FORMAT: TextureFormat = TextureFormat::Rgba32Float;
pub const CLARITY_GUIDE_KERNEL_HALF: usize = 12;
pub const CLARITY_GUIDE_WGSL: &str = include_str!("../../../assets/shaders/clarity_guide.wgsl");

pub fn presence_adjust_wgsl() -> String {
    include_str!("../../../assets/shaders/presence_adjust.wgsl")
        .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl())
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PresenceParams {
    pub size: [u32; 2],
    pub _pad0: [u32; 2],
    pub amounts: [f32; 4],
    pub mips: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ClarityGuideParams {
    pub size: [u32; 2],
    pub radius: u32,
    pub mode: u32,
    pub eps: f32,
    pub _pad: [u32; 3],
    pub weights: [f32; CLARITY_GUIDE_KERNEL_HALF],
}

pub struct PresencePass {
    pub adjust_layout: BindGroupLayout,
    pub adjust_pipeline: ComputePipeline,
    pub guide_layout: BindGroupLayout,
    pub guide_pipeline: ComputePipeline,
}

impl PresencePass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let adjust_layout = make_layout(
            ctx,
            "presence-adjust-bgl",
            &[
                uniform_entry(0, PRESENCE_UNIFORM_SIZE),
                tex_entry(1),
                tex_entry(2),
                storage_entry(3, ctx.linear_format),
                tex_entry_with(4, false, TextureViewDimension::D2),
            ],
        );
        let adjust_pipeline = make_pipeline(
            ctx,
            &adjust_layout,
            "presence_adjust.wgsl",
            &presence_adjust_wgsl(),
        );
        let guide_layout = make_layout(
            ctx,
            "clarity-guide-bgl",
            &[
                uniform_entry(0, CLARITY_GUIDE_UNIFORM_SIZE),
                tex_entry_with(1, false, TextureViewDimension::D2),
                storage_entry(2, CLARITY_GUIDE_FORMAT),
            ],
        );
        let guide_pipeline =
            make_pipeline(ctx, &guide_layout, "clarity_guide.wgsl", CLARITY_GUIDE_WGSL);
        Self {
            adjust_layout,
            adjust_pipeline,
            guide_layout,
            guide_pipeline,
        }
    }
}
