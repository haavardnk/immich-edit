use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const PARAMS_BYTES: usize = size_of::<ResampleParams>();

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ResampleParams {
    pub dst_size: [u32; 2],
    pub src_size: [u32; 2],
    pub scale: f32,
    pub inv_filter_scale: f32,
    pub axis: u32,
    pub _pad: u32,
}

pub struct ResamplePass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl ResamplePass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "resample-bgl",
            &[
                uniform_entry(0, PARAMS_BYTES as u64),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "resample.wgsl",
            include_str!("../../../assets/shaders/resample.wgsl"),
        );
        Self { layout, pipeline }
    }
}

pub fn pack_params(
    dst_size: (u32, u32),
    src_size: (u32, u32),
    scale: f32,
    axis: u32,
) -> ResampleParams {
    ResampleParams {
        dst_size: [dst_size.0, dst_size.1],
        src_size: [src_size.0, src_size.1],
        scale,
        inv_filter_scale: 1.0 / scale.max(1.0),
        axis,
        _pad: 0,
    }
}
