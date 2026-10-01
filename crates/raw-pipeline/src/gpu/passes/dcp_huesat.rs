use std::sync::Arc;

use wgpu::{
    BindGroupLayout, ComputePipeline, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
};

use crate::dcp::HueSatMap;
use crate::gpu::context::GpuContext;
use crate::ops::curves::{DISPLAY_CURVES_UNIFORM_SIZE, display_curves_wgsl};

use super::common::{make_layout, make_pipeline_raw, storage_entry, tex_entry_with, uniform_entry};

pub const DCP_HUESAT_UNIFORM_SIZE: u64 = size_of::<DcpHueSatParams>() as u64;

const HUESAT_TABLE_WGSL: &str = include_str!("../../../assets/shaders/huesat_table.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DcpHueSatParams {
    pub dims: [u32; 4],
    pub to_pp: [[f32; 4]; 3],
    pub from_pp: [[f32; 4]; 3],
    pub flags: [u32; 4],
    pub tone_lut: [[f32; 4]; 64],
}

pub fn dcp_huesat_wgsl(out_format: wgpu::TextureFormat) -> String {
    include_str!("../../../assets/shaders/dcp_huesat.wgsl")
        .replace(
            crate::gpu::display_depth::DISPLAY_STORE_INJECT,
            &crate::gpu::display_depth::store_wgsl(out_format, 3),
        )
        .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl())
        .replace("// DISPLAY_CURVES_INJECT", &display_curves_wgsl(4))
        .replace("// HUESAT_TABLE_INJECT", HUESAT_TABLE_WGSL)
}

pub fn upload_huesat_table(ctx: &GpuContext, map: &HueSatMap) -> Texture {
    let size = Extent3d {
        width: map.hue_div,
        height: map.sat_div,
        depth_or_array_layers: map.val_div.max(1),
    };
    let rgba: Vec<f32> = map
        .data
        .iter()
        .flat_map(|px| [px[0], px[1], px[2], 0.0])
        .collect();
    let tex = ctx.device.create_texture(&TextureDescriptor {
        label: Some("dcp-huesat-3d"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D3,
        format: wgpu::TextureFormat::Rgba32Float,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    });
    ctx.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        bytemuck::cast_slice(&rgba),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(map.hue_div * 16),
            rows_per_image: Some(map.sat_div),
        },
        size,
    );
    tex
}

pub fn huesat_table_view(texture: &Texture) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D3),
        ..Default::default()
    })
}

pub struct DcpHueSatPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl DcpHueSatPass {
    pub fn new(ctx: &Arc<GpuContext>, out_format: wgpu::TextureFormat) -> Self {
        let layout = make_layout(
            ctx,
            "dcp-look-bgl",
            &[
                uniform_entry(0, DCP_HUESAT_UNIFORM_SIZE),
                tex_entry_with(1, false, TextureViewDimension::D2),
                tex_entry_with(2, false, TextureViewDimension::D3),
                storage_entry(3, out_format),
                uniform_entry(4, DISPLAY_CURVES_UNIFORM_SIZE),
            ],
        );
        let src = dcp_huesat_wgsl(out_format);
        let pipeline = make_pipeline_raw(ctx, &layout, "dcp-look-cp", &src);

        Self { layout, pipeline }
    }
}
