// color-space: X-Trans mosaic f32 in → linear scene-referred Rgba16Float out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_buffer_entry, storage_buffer_entry_rw, storage_entry,
    uniform_entry_unsized,
};

pub struct XtransPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

pub struct XtransPasses {
    pub green: XtransPass,
    pub rgb: XtransPass,
}

impl XtransPasses {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let green_layout = make_layout(
            ctx,
            "xtrans-green-bgl",
            &[
                uniform_entry_unsized(0),
                storage_buffer_entry(1),
                storage_buffer_entry_rw(2),
            ],
        );
        let green_pipeline = make_pipeline(
            ctx,
            &green_layout,
            "xtrans_green.wgsl",
            include_str!("../../../assets/shaders/xtrans_green.wgsl"),
        );

        let rgb_layout = make_layout(
            ctx,
            "xtrans-rgb-bgl",
            &[
                uniform_entry_unsized(0),
                storage_buffer_entry(1),
                storage_buffer_entry(2),
                storage_entry(3, ctx.linear_format),
            ],
        );
        let rgb_pipeline = make_pipeline(
            ctx,
            &rgb_layout,
            "xtrans_rgb.wgsl",
            include_str!("../../../assets/shaders/xtrans_rgb.wgsl"),
        );

        Self {
            green: XtransPass {
                layout: green_layout,
                pipeline: green_pipeline,
            },
            rgb: XtransPass {
                layout: rgb_layout,
                pipeline: rgb_pipeline,
            },
        }
    }
}
