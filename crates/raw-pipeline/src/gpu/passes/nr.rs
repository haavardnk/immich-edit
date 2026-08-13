// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const NR_UNIFORM_SIZE: u64 = size_of::<NrParams>() as u64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NrParams {
    pub size: [u32; 2],
    pub radius: u32,
    pub stage: u32,
    pub inv_2ss: f32,
    pub inv_2sr_luma: f32,
    pub inv_2sr_chroma: f32,
    pub alpha_luma: f32,
    pub alpha_chroma: f32,
    pub contrast: f32,
    pub _pad: [f32; 2],
}

pub struct NrPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl NrPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "nr-bgl",
            &[
                uniform_entry(0, NR_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "nr.wgsl",
            include_str!("../../../assets/shaders/nr.wgsl"),
        );
        Self { layout, pipeline }
    }
}
