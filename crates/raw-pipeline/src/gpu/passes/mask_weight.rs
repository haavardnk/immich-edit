// color-space: linear scene-referred Rgba16Float in → R16Float weight out
use std::collections::HashMap;
use std::sync::Arc;

use wgpu::{
    AddressMode, BindGroupLayout, ComputePipeline, FilterMode, Sampler, SamplerDescriptor,
    TextureFormat, TextureViewDimension,
};

use crate::gpu::context::GpuContext;
use crate::mask_raster::MaskRaster;

use super::common::{
    make_layout, make_pipeline_raw, sampler_entry, storage_buffer_entry, storage_entry, tex_entry,
    tex_entry_with, uniform_entry_unsized,
};

pub const COMPONENT_BYTES: usize = size_of::<MaskComponent>();
pub const MAX_COMPONENTS: usize = 32;
pub const MAX_COMPONENTS_BYTES: usize = COMPONENT_BYTES * MAX_COMPONENTS;
pub const PARAMS_BYTES: usize = size_of::<MaskWeightParams>();
pub const ATLAS_DIM: u32 = 1024;
pub const ATLAS_LAYERS: u32 = 16;
pub const MAX_POLY_VERTS: usize = MAX_COMPONENTS * crate::edits::N_MAX_POLYGON_POINTS;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaskComponent {
    pub kind: u32,
    pub mode: u32,
    pub invert: u32,
    pub slot: u32,
    pub geom_a: [f32; 4],
    pub geom_b: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaskWeightParams {
    pub out_size: [u32; 2],
    pub n_components: u32,
    pub layer_amount: f32,
    pub crop: [f32; 4],
    pub flags: [u32; 4],
    pub geom_extra2: [f32; 4],
    pub geom_extra3: [f32; 4],
    pub lens: [f32; 4],
    pub perspective: [f32; 12],
}

const SHADER: &str = include_str!("../../../assets/shaders/mask_weight.wgsl");

pub struct MaskWeightPass {
    pub layout: BindGroupLayout,
    pub pipeline: ComputePipeline,
}

impl MaskWeightPass {
    pub fn new(ctx: &Arc<GpuContext>) -> Self {
        let layout = make_layout(
            ctx,
            "mask-weight-bgl",
            &[
                uniform_entry_unsized(0),
                storage_buffer_entry(1),
                storage_entry(2, TextureFormat::R32Float),
                tex_entry_with(3, true, TextureViewDimension::D2Array),
                sampler_entry(4),
                tex_entry(5),
                storage_buffer_entry(6),
            ],
        );
        let source = format!("{}\n{}", crate::gpu::shader_builder::GEOMETRY_WGSL, SHADER)
            .replace("// TONE_WGSL_INJECT", crate::tone::wgsl::tone_wgsl());
        let pipeline = make_pipeline_raw(ctx, &layout, "mask-weight-cp", &source);
        Self { layout, pipeline }
    }
}

pub fn make_atlas_sampler(ctx: &Arc<GpuContext>) -> Sampler {
    ctx.device.create_sampler(&SamplerDescriptor {
        label: Some("mask-atlas-sampler"),
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        ..Default::default()
    })
}

pub fn resample_raster_to_atlas(raster: &MaskRaster) -> Vec<u8> {
    let dim = ATLAS_DIM as usize;
    let mut out = vec![0u8; dim * dim];
    let inv = 1.0 / dim as f32;
    for y in 0..dim {
        let v = (y as f32 + 0.5) * inv;
        let row = y * dim;
        for x in 0..dim {
            let u = (x as f32 + 0.5) * inv;
            let w = raster.sample_bilinear(u, v).clamp(0.0, 1.0);
            out[row + x] = (w * 255.0 + 0.5) as u8;
        }
    }
    out
}

pub fn pack_layer_eval(
    layer: &crate::cpu::masked::LayerEval,
    slot_map: &HashMap<String, u32>,
) -> (Vec<u8>, u32, Vec<u8>) {
    let mut out = Vec::with_capacity(layer.components.len() * COMPONENT_BYTES);
    let mut verts: Vec<u8> = Vec::new();
    let mut vert_count: usize = 0;
    let mut n: u32 = 0;
    for c in &layer.components {
        if n as usize >= MAX_COMPONENTS {
            break;
        }
        let mut slot: u32 = 0;
        let (kind, geom_a, geom_b) = match &c.kind {
            crate::cpu::masked::ComponentKindEval::Linear {
                p0,
                dir,
                len2,
                feather,
            } => (
                0u32,
                [p0.0, p0.1, dir.0, dir.1],
                [*len2, *feather, 0.0, 0.0],
            ),
            crate::cpu::masked::ComponentKindEval::Radial {
                center,
                inv_radius,
                feather,
            } => (
                1u32,
                [center.0, center.1, inv_radius.0, inv_radius.1],
                [0.0, *feather, 0.0, 0.0],
            ),
            crate::cpu::masked::ComponentKindEval::Brush { raster_id, raster } => {
                if raster.is_none() {
                    continue;
                }
                let Some(s) = slot_map.get(raster_id) else {
                    continue;
                };
                slot = *s;
                (2u32, [0.0, 0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 0.0])
            }
            crate::cpu::masked::ComponentKindEval::LumaRange { min, max, softness } => {
                (3u32, [*min, *max, *softness, 0.0], [0.0, 0.0, 0.0, 0.0])
            }
            crate::cpu::masked::ComponentKindEval::ColorRange {
                sample_rgb: _,
                sample_lab,
                tolerance,
                softness,
            } => (
                4u32,
                [sample_lab[0], sample_lab[1], sample_lab[2], 0.0],
                [*tolerance, *softness, 0.0, 0.0],
            ),
            crate::cpu::masked::ComponentKindEval::Polygon { points, feather } => {
                let take = points
                    .len()
                    .min(crate::edits::N_MAX_POLYGON_POINTS)
                    .min(MAX_POLY_VERTS.saturating_sub(vert_count));
                if take < 3 {
                    continue;
                }
                let offset = vert_count as f32;
                for p in points.iter().take(take) {
                    verts.extend_from_slice(&p.0.to_ne_bytes());
                    verts.extend_from_slice(&p.1.to_ne_bytes());
                }
                vert_count += take;
                (
                    5u32,
                    [offset, take as f32, *feather, 0.0],
                    [0.0, 0.0, 0.0, 0.0],
                )
            }
        };
        let mode = match c.mode {
            crate::edits::MaskComponentMode::Add => 0u32,
            crate::edits::MaskComponentMode::Subtract => 1u32,
            crate::edits::MaskComponentMode::Intersect => 2u32,
        };
        let invert = if c.invert { 1u32 } else { 0u32 };
        let component = MaskComponent {
            kind,
            mode,
            invert,
            slot,
            geom_a,
            geom_b,
        };
        out.extend_from_slice(bytemuck::bytes_of(&component));
        n += 1;
    }
    (out, n, verts)
}
