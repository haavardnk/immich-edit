// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_buffer_entry, storage_entry, tex_entry, uniform_entry,
};

pub const RETOUCH_UNIFORM_SIZE: u64 = 64;

pub struct RetouchPasses {
    pub prep_layout: BindGroupLayout,
    pub prep_pipeline: ComputePipeline,
    pub blur_layout: BindGroupLayout,
    pub blur_pipeline: ComputePipeline,
    pub apply_layout: BindGroupLayout,
    pub apply_pipeline: ComputePipeline,
}

impl RetouchPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let prep_layout = make_layout(
            ctx,
            "retouch-prep-bgl",
            &[
                uniform_entry(0, RETOUCH_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
                storage_entry(3, ctx.linear_format),
            ],
        );
        let prep_pipeline = make_pipeline(
            ctx,
            &prep_layout,
            "retouch_prep.wgsl",
            include_str!("../../../assets/shaders/retouch_prep.wgsl"),
        );

        let blur_layout = make_layout(
            ctx,
            "retouch-blur-bgl",
            &[
                uniform_entry(0, RETOUCH_UNIFORM_SIZE),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let blur_pipeline = make_pipeline(
            ctx,
            &blur_layout,
            "retouch_blur.wgsl",
            include_str!("../../../assets/shaders/retouch_blur.wgsl"),
        );

        let apply_layout = make_layout(
            ctx,
            "retouch-apply-bgl",
            &[
                uniform_entry(0, RETOUCH_UNIFORM_SIZE),
                tex_entry(1),
                tex_entry(2),
                tex_entry(3),
                storage_buffer_entry(4),
                storage_entry(5, ctx.linear_format),
            ],
        );
        let apply_pipeline = make_pipeline(
            ctx,
            &apply_layout,
            "retouch_apply.wgsl",
            include_str!("../../../assets/shaders/retouch_apply.wgsl"),
        );

        Self {
            prep_layout,
            prep_pipeline,
            blur_layout,
            blur_pipeline,
            apply_layout,
            apply_pipeline,
        }
    }
}
