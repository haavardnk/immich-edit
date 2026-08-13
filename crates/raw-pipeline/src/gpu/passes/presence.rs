// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const PRESENCE_UNIFORM_SIZE: u64 = size_of::<PresenceParams>() as u64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PresenceParams {
    pub size: [u32; 2],
    pub _pad0: [u32; 2],
    pub amounts: [f32; 4],
    pub mips: [u32; 4],
}

pub struct PresencePass {
    pub adjust_layout: BindGroupLayout,
    pub adjust_pipeline: ComputePipeline,
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
            ],
        );
        let adjust_pipeline = make_pipeline(
            ctx,
            &adjust_layout,
            "presence_adjust.wgsl",
            include_str!("../../../assets/shaders/presence_adjust.wgsl"),
        );
        Self {
            adjust_layout,
            adjust_pipeline,
        }
    }
}
