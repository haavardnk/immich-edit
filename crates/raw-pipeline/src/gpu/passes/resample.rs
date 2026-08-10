use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry};

pub const PARAMS_BYTES: usize = 32;

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
) -> [u8; PARAMS_BYTES] {
    let filter_scale = scale.max(1.0);
    let mut out = [0u8; PARAMS_BYTES];
    out[0..4].copy_from_slice(&dst_size.0.to_le_bytes());
    out[4..8].copy_from_slice(&dst_size.1.to_le_bytes());
    out[8..12].copy_from_slice(&src_size.0.to_le_bytes());
    out[12..16].copy_from_slice(&src_size.1.to_le_bytes());
    out[16..20].copy_from_slice(&scale.to_le_bytes());
    out[20..24].copy_from_slice(&(1.0 / filter_scale).to_le_bytes());
    out[24..28].copy_from_slice(&axis.to_le_bytes());
    out
}
