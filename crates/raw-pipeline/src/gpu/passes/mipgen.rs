use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry};

pub struct MipgenPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MipgenPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "mipgen-bgl",
            &[tex_entry(0), storage_entry(1, ctx.linear_format)],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "mipgen.wgsl",
            include_str!("../../../assets/shaders/mipgen.wgsl"),
        );
        Self { layout, pipeline }
    }
}
