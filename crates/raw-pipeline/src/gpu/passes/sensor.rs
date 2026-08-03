// color-space: linear scene-referred Rgba16Float in/out
use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline, storage_entry, tex_entry, uniform_entry_unsized};

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
        let pipeline = make_pipeline(
            ctx,
            &layout,
            "sensor.wgsl",
            include_str!("../../../assets/shaders/sensor.wgsl"),
        );
        Self { layout, pipeline }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SensorParams {
    pub size: [u32; 2],
    pub zoom: f32,
    pub vig_amount: f32,
    pub coeffs: [f32; 4],
    pub ca_vig: [f32; 4],
}

impl SensorParams {
    pub fn from_edits(lens: &crate::edits::LensEdits, w: u32, h: u32) -> Self {
        let (k1, k2, k3) = crate::ops::lens_distortion::distortion_coeffs(lens);
        let (red_scale, blue_scale) = crate::ops::lens_ca::ca_scales(lens);
        let (vk1, vk2, vk3, vig_amount) = crate::ops::lens_vignette::vignette_coeffs(lens);
        let zoom = crate::ops::lens_distortion::distortion_zoom(lens);
        Self {
            size: [w, h],
            zoom,
            vig_amount,
            coeffs: [k1, k2, k3, vk1],
            ca_vig: [red_scale, blue_scale, vk2, vk3],
        }
    }
}
