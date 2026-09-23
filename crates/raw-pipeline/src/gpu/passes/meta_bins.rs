// color-space: display-referred texture and scene-linear output in, bin counts out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureViewDimension};

use crate::gpu::context::GpuContext;
use crate::gpu::display_depth::{DISPLAY_LOAD_INJECT, DisplayDepth};
use crate::histogram::BINS;

use super::common::{
    display_src_entry, make_layout, make_pipeline_raw, storage_buffer_entry_rw, tex_entry_with,
    uniform_entry,
};

pub const HISTOGRAM_TILE: u32 = 64;
pub const HISTOGRAM_COUNTS: usize = 2 * 4 * BINS;
pub const SCOPES_GROUP: u32 = 16;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BinParams {
    pub size: [u32; 2],
    pub step: u32,
    pub _pad: u32,
}

pub struct MetaBinsPasses {
    pub histogram_layout: BindGroupLayout,
    pub histogram: ComputePipeline,
    pub scopes_layout: BindGroupLayout,
    pub scopes: ComputePipeline,
}

impl MetaBinsPasses {
    pub fn new(ctx: &Arc<GpuContext>, depth: DisplayDepth) -> Self {
        let params = uniform_entry(0, size_of::<BinParams>() as u64);
        let histogram_layout = make_layout(
            ctx,
            "histogram-bgl",
            &[
                params,
                display_src_entry(1, depth),
                tex_entry_with(2, false, TextureViewDimension::D2),
                storage_buffer_entry_rw(3),
            ],
        );
        let histogram = make_pipeline_raw(
            ctx,
            &histogram_layout,
            "histogram.wgsl",
            &include_str!("../../../assets/shaders/histogram.wgsl")
                .replace(DISPLAY_LOAD_INJECT, &depth.load_u8_wgsl(1)),
        );
        let scopes_layout = make_layout(
            ctx,
            "scopes-bgl",
            &[
                params,
                display_src_entry(1, depth),
                storage_buffer_entry_rw(2),
            ],
        );
        let scopes = make_pipeline_raw(
            ctx,
            &scopes_layout,
            "scopes.wgsl",
            &include_str!("../../../assets/shaders/scopes.wgsl")
                .replace(DISPLAY_LOAD_INJECT, &depth.load_u8_wgsl(1)),
        );
        Self {
            histogram_layout,
            histogram,
            scopes_layout,
            scopes,
        }
    }
}
