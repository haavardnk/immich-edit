// color-space: Bayer u16 in → linear scene-referred Rgba16Float out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureFormat};

use crate::gpu::context::GpuContext;

use super::common::{
    make_layout, make_pipeline, storage_buffer_entry, storage_entry, uniform_entry_unsized,
};

pub struct DemosaicPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DemosaicPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "demosaic-bgl",
            &[
                uniform_entry_unsized(0),
                storage_buffer_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "demosaic.wgsl",
            include_str!("../../../assets/shaders/demosaic.wgsl"),
        );
        Self { layout, pipeline }
    }
}

pub(super) fn linear_format_str(fmt: TextureFormat) -> &'static str {
    match fmt {
        TextureFormat::Rgba16Float => "rgba16float",
        TextureFormat::Rgba32Float => "rgba32float",
        _ => "rgba16float",
    }
}
