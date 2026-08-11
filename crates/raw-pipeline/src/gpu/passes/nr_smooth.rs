// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const NR_SMOOTH_UNIFORM_SIZE: u64 = 48;

pub struct NrSmoothPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl NrSmoothPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "nr-smooth-bgl",
            &[
                uniform_entry(0, NR_SMOOTH_UNIFORM_SIZE),
                tex_entry(1),
                tex_entry(2),
                storage_entry(3, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "nr_smooth.wgsl",
            include_str!("../../../assets/shaders/nr_smooth.wgsl"),
        );
        Self { layout, pipeline }
    }
}
