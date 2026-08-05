use std::sync::Arc;

use wgpu::{BindGroupLayout, ComputePipeline, TextureViewDimension};

use crate::gpu::context::GpuContext;

use super::common::{make_layout, make_pipeline_raw, storage_entry, tex_entry_with, uniform_entry};

pub const DCP_HUESAT_UNIFORM_SIZE: u64 = 1152;

pub struct DcpHueSatPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DcpHueSatPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        Self::with_format(ctx, wgpu::TextureFormat::Rgba16Float, "dcp-huesat")
    }

    pub fn new_look(ctx: &Arc<GpuContext>) -> Self {
        Self::with_format(ctx, wgpu::TextureFormat::Rgba8Unorm, "dcp-look")
    }

    fn with_format(ctx: &Arc<GpuContext>, out_format: wgpu::TextureFormat, label: &str) -> Self {
        let layout = make_layout(
            ctx,
            &format!("{label}-bgl"),
            &[
                uniform_entry(0, DCP_HUESAT_UNIFORM_SIZE),
                tex_entry_with(1, false, TextureViewDimension::D2),
                tex_entry_with(2, false, TextureViewDimension::D3),
                storage_entry(3, out_format),
            ],
        );
        let src = include_str!("../../../assets/shaders/dcp_huesat.wgsl")
            .replace("rgba16float", storage_format_str(out_format))
            .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl());
        let pipeline = make_pipeline_raw(ctx, &layout, &format!("{label}-cp"), &src);

        Self { layout, pipeline }
    }
}

fn storage_format_str(f: wgpu::TextureFormat) -> &'static str {
    match f {
        wgpu::TextureFormat::Rgba8Unorm => "rgba8unorm",
        _ => "rgba16float",
    }
}
