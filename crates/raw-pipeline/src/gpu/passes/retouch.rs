// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_buffer_entry, storage_entry, tex_entry, uniform_entry,
};

pub const RETOUCH_UNIFORM_SIZE: u64 = size_of::<RetouchParams>() as u64;
pub const RETOUCH_PREP_WGSL: &str = include_str!("../../../assets/shaders/retouch_prep.wgsl");
pub const RETOUCH_PUSH_WGSL: &str = include_str!("../../../assets/shaders/retouch_push.wgsl");
pub const RETOUCH_PULL_WGSL: &str = include_str!("../../../assets/shaders/retouch_pull.wgsl");
pub const RETOUCH_APPLY_WGSL: &str = include_str!("../../../assets/shaders/retouch_apply.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RetouchParams {
    pub dims: [u32; 2],
    pub bbox_origin: [u32; 2],
    pub bbox_size: [u32; 2],
    pub point_count: u32,
    pub clone_mode: u32,
    pub offset: [f32; 2],
    pub radius_px: f32,
    pub hardness: f32,
    pub opacity: f32,
    pub _pad: [f32; 3],
}

pub struct RetouchPasses {
    pub prep_layout: BindGroupLayout,
    pub prep_pipeline: ComputePipeline,
    pub push_layout: BindGroupLayout,
    pub push_pipeline: ComputePipeline,
    pub pull_layout: BindGroupLayout,
    pub pull_pipeline: ComputePipeline,
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
                storage_buffer_entry(4),
            ],
        );
        let prep_pipeline =
            make_pipeline(ctx, &prep_layout, "retouch_prep.wgsl", RETOUCH_PREP_WGSL);

        let push_layout = make_layout(
            ctx,
            "retouch-push-bgl",
            &[tex_entry(0), storage_entry(1, ctx.linear_format)],
        );
        let push_pipeline =
            make_pipeline(ctx, &push_layout, "retouch_push.wgsl", RETOUCH_PUSH_WGSL);

        let pull_layout = make_layout(
            ctx,
            "retouch-pull-bgl",
            &[
                tex_entry(0),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let pull_pipeline =
            make_pipeline(ctx, &pull_layout, "retouch_pull.wgsl", RETOUCH_PULL_WGSL);

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
        let apply_pipeline =
            make_pipeline(ctx, &apply_layout, "retouch_apply.wgsl", RETOUCH_APPLY_WGSL);

        Self {
            prep_layout,
            prep_pipeline,
            push_layout,
            push_pipeline,
            pull_layout,
            pull_pipeline,
            apply_layout,
            apply_pipeline,
        }
    }
}
