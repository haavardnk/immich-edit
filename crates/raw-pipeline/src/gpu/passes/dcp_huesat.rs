use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureViewDimension};

use crate::gpu::context::GpuContext;
use crate::ops::curves::{DISPLAY_CURVES_UNIFORM_SIZE, display_curves_wgsl};

use super::common::{make_layout, make_pipeline_raw, storage_entry, tex_entry_with, uniform_entry};

pub const DCP_HUESAT_UNIFORM_SIZE: u64 = size_of::<DcpHueSatParams>() as u64;

const NO_DISPLAY_CURVES_WGSL: &str =
    "fn display_curves_apply(c: vec3<f32>) -> vec3<f32> { return c; }";

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DcpHueSatParams {
    pub dims: [u32; 4],
    pub to_pp: [[f32; 4]; 3],
    pub from_pp: [[f32; 4]; 3],
    pub flags: [u32; 4],
    pub tone_lut: [[f32; 4]; 64],
}

pub fn dcp_huesat_wgsl(out_format: wgpu::TextureFormat, output: bool) -> String {
    let curves = if output {
        display_curves_wgsl(4)
    } else {
        NO_DISPLAY_CURVES_WGSL.to_owned()
    };
    include_str!("../../../assets/shaders/dcp_huesat.wgsl")
        .replace(
            crate::gpu::display_depth::DISPLAY_STORE_INJECT,
            &crate::gpu::display_depth::store_wgsl(out_format, 3),
        )
        .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl())
        .replace("// DISPLAY_CURVES_INJECT", &curves)
}

pub struct DcpHueSatPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DcpHueSatPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        Self::with_format(ctx, wgpu::TextureFormat::Rgba16Float, false, "dcp-huesat")
    }

    pub fn new_look(ctx: &Arc<GpuContext>, out_format: wgpu::TextureFormat) -> Self {
        Self::with_format(ctx, out_format, true, "dcp-look")
    }

    fn with_format(
        ctx: &Arc<GpuContext>,
        out_format: wgpu::TextureFormat,
        output: bool,
        label: &str,
    ) -> Self {
        let mut entries = vec![
            uniform_entry(0, DCP_HUESAT_UNIFORM_SIZE),
            tex_entry_with(1, false, TextureViewDimension::D2),
            tex_entry_with(2, false, TextureViewDimension::D3),
            storage_entry(3, out_format),
        ];
        if output {
            entries.push(uniform_entry(4, DISPLAY_CURVES_UNIFORM_SIZE));
        }
        let layout = make_layout(ctx, &format!("{label}-bgl"), &entries);
        let src = dcp_huesat_wgsl(out_format, output);
        let pipeline = make_pipeline_raw(ctx, &layout, &format!("{label}-cp"), &src);

        Self { layout, pipeline }
    }
}
