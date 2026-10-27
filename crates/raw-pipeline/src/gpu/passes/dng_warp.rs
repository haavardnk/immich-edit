// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::geom::dng_warp::DngWarp;
use crate::gpu::context::GpuContext;

use super::common::{
    BICUBIC_CLAMPED_WGSL, make_layout, make_pipeline, storage_entry, tex_entry,
    uniform_entry_unsized,
};

const DNG_WARP_WGSL: &str = include_str!("../../../assets/shaders/dng_warp.wgsl");

pub fn dng_warp_wgsl() -> String {
    [DNG_WARP_WGSL, BICUBIC_CLAMPED_WGSL].concat()
}

pub struct DngWarpPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DngWarpPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "dng-warp-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(ctx, &layout, "dng_warp.wgsl", &dng_warp_wgsl());
        Self { layout, pipeline }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DngWarpParams {
    pub size: [u32; 2],
    pub center: [f32; 2],
    pub radius: f32,
    pub split: u32,
    pub _pad: [u32; 2],
    pub radial: [[f32; 4]; 3],
    pub tangential: [[f32; 4]; 3],
}

impl DngWarpParams {
    pub fn new(warp: &DngWarp, w: u32, h: u32) -> Self {
        let frame = warp.frame(w as usize, h as usize);
        Self {
            size: [w, h],
            center: frame.center,
            radius: frame.radius,
            split: u32::from(warp.is_split()),
            _pad: [0; 2],
            radial: warp.planes.map(|p| p.radial),
            tangential: warp
                .planes
                .map(|p| [p.tangential[0], p.tangential[1], 0.0, 0.0]),
        }
    }
}
