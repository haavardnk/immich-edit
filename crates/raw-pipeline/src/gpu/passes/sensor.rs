// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::edits::LensEdits;
use crate::gpu::context::GpuContext;
use crate::tile::Rect;

use super::common::{
    BICUBIC_CLAMPED_WGSL, make_layout, make_pipeline, storage_entry, tex_entry,
    uniform_entry_unsized,
};

const SENSOR_WGSL: &str = include_str!("../../../assets/shaders/sensor.wgsl");

pub fn sensor_wgsl() -> String {
    [SENSOR_WGSL, BICUBIC_CLAMPED_WGSL].concat()
}

pub struct SensorPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl SensorPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "sensor-bgl",
            &[
                uniform_entry_unsized(0),
                tex_entry(1),
                storage_entry(2, ctx.linear_format),
            ],
        );
        let pipeline = make_pipeline(ctx, &layout, "sensor.wgsl", &sensor_wgsl());
        Self { layout, pipeline }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SensorParams {
    pub frame: [u32; 2],
    pub zoom: f32,
    pub vig_amount: f32,
    pub coeffs: [f32; 4],
    pub ca_vig: [f32; 4],
    pub dst_origin: [u32; 2],
    pub dst_size: [u32; 2],
    pub src_origin: [i32; 2],
    pub _pad: [u32; 2],
}

impl SensorParams {
    pub fn new(lens: &LensEdits, (w, h): (u32, u32), src: Rect, dst: Rect) -> Self {
        let (k1, k2, k3) = crate::ops::lens_distortion::distortion_coeffs(lens);
        let (red_scale, blue_scale) = crate::ops::lens_ca::ca_scales(lens);
        let (vk1, vk2, vk3, vig_amount) = crate::ops::lens_vignette::vignette_coeffs(lens);
        let zoom = crate::ops::lens_distortion::distortion_zoom(lens);
        Self {
            frame: [w, h],
            zoom,
            vig_amount,
            coeffs: [k1, k2, k3, vk1],
            ca_vig: [red_scale, blue_scale, vk2, vk3],
            dst_origin: [dst.x, dst.y],
            dst_size: [dst.w, dst.h],
            src_origin: [src.x as i32, src.y as i32],
            _pad: [0; 2],
        }
    }
}
