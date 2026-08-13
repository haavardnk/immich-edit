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

const SHADER: &str = r#"
struct MaskParams {
    out_size: vec2<u32>,
    n_components: u32,
    layer_amount: f32,
    crop: vec4<f32>,
    flags: vec4<u32>,
    geom_extra2: vec4<f32>,
    geom_extra3: vec4<f32>,
    lens: vec4<f32>,
    persp0: vec4<f32>,
    persp1: vec4<f32>,
    persp2: vec4<f32>,
};

struct Component {
    kind_mode_invert_pad: vec4<u32>,
    geom_a: vec4<f32>,
    geom_b: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: MaskParams;
@group(0) @binding(1) var<storage, read> comps: array<Component>;
@group(0) @binding(2) var weight_out: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var atlas: texture_2d_array<f32>;
@group(0) @binding(4) var samp: sampler;
@group(0) @binding(5) var display_tex: texture_2d<f32>;
@group(0) @binding(6) var<storage, read> poly: array<vec2<f32>>;

// TONE_WGSL_INJECT

fn smoothstep_calc(e0: f32, e1: f32, x: f32) -> f32 {
    let t = clamp((x - e0) / max(e1 - e0, 1e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn srgb_to_linear(v: f32) -> f32 {
    let c = clamp(v, 0.0, 1.0);
    if (c <= 0.04045) { return c / 12.92; }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn display_srgb_to_oklab(rgb: vec3<f32>) -> vec3<f32> {
    let lin = vec3<f32>(
        srgb_to_linear(rgb.x),
        srgb_to_linear(rgb.y),
        srgb_to_linear(rgb.z),
    );
    let l = pow(0.41222146 * lin.x + 0.53633255 * lin.y + 0.051445995 * lin.z, 1.0 / 3.0);
    let m = pow(0.2119035 * lin.x + 0.6806995 * lin.y + 0.10739696 * lin.z, 1.0 / 3.0);
    let s = pow(0.08830246 * lin.x + 0.28171885 * lin.y + 0.6299787 * lin.z, 1.0 / 3.0);
    return vec3<f32>(
        0.21045426 * l + 0.7936178 * m - 0.004072047 * s,
        1.9779985 * l - 2.4285922 * m + 0.4505937 * s,
        0.025904037 * l + 0.78277177 * m - 0.80867577 * s,
    );
}

fn luma_range_weight(luma: f32, lo: f32, hi: f32, softness: f32) -> f32 {
    if (softness <= 1e-6) {
        if (luma >= lo && luma <= hi) { return 1.0; }
        return 0.0;
    }
    let lower = smoothstep_calc(lo - softness, lo, luma);
    let upper = 1.0 - smoothstep_calc(hi, hi + softness, luma);
    return lower * upper;
}

fn color_range_weight(
    rgb: vec3<f32>,
    sample_lab: vec3<f32>,
    tolerance: f32,
    softness: f32,
) -> f32 {
    let lab = display_srgb_to_oklab(rgb);
    let distance = length(lab - sample_lab);
    if (softness <= 1e-6) {
        if (distance <= tolerance) { return 1.0; }
        return 0.0;
    }
    return 1.0 - smoothstep_calc(tolerance, tolerance + softness, distance);
}

fn point_segment_distance(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let d = b - a;
    let len2 = dot(d, d);
    var t = 0.0;
    if (len2 > 1e-12) {
        t = clamp(dot(p - a, d) / len2, 0.0, 1.0);
    }
    return length(p - (a + t * d));
}

fn polygon_weight(offset: u32, count: u32, uv: vec2<f32>, feather: f32) -> f32 {
    if (count < 3u) { return 0.0; }
    var inside = false;
    var nearest = 1e30;
    var j = count - 1u;
    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let pi = poly[offset + i];
        let pj = poly[offset + j];
        if ((pi.y > uv.y) != (pj.y > uv.y)) {
            let t = (uv.y - pi.y) / (pj.y - pi.y);
            if (uv.x < pi.x + t * (pj.x - pi.x)) { inside = !inside; }
        }
        nearest = min(nearest, point_segment_distance(uv, pi, pj));
        j = i;
    }
    if (!inside) { return 0.0; }
    if (feather <= 1e-6) { return 1.0; }
    return smoothstep_calc(0.0, feather, nearest);
}

fn display_to_scene(disp_u: f32, disp_v: f32) -> vec2<f32> {
    let oriented = geom_display_to_oriented(
        p.crop,
        p.geom_extra2,
        p.geom_extra3,
        p.persp0,
        p.persp1,
        p.persp2,
        vec2<f32>(disp_u, disp_v),
    );
    let m = geom_ortho_inverse(p.flags, oriented);
    let mu = m.x;
    let mv = m.y;
    let k1 = p.lens.x;
    let k2 = p.lens.y;
    let k3 = p.lens.z;
    let zoom = p.lens.w;
    if (k1 == 0.0 && k2 == 0.0 && k3 == 0.0 && zoom == 1.0) {
        return vec2<f32>(mu, mv);
    }
    let mw = p.geom_extra3.z;
    let mh = p.geom_extra3.w;
    let half_diag = 0.5 * sqrt(mw * mw + mh * mh);
    let nx = (mu - 0.5) * mw;
    let ny = (mv - 0.5) * mh;
    let r = sqrt(nx * nx + ny * ny) * zoom / max(half_diag, 1e-6);
    let r2 = r * r;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
    return vec2<f32>(0.5 + (mu - 0.5) * zoom * s, 0.5 + (mv - 0.5) * zoom * s);
}

fn component_weight(c: Component, u: f32, v: f32, display_rgb: vec3<f32>) -> f32 {
    var raw: f32 = 0.0;
    let kind = c.kind_mode_invert_pad.x;
    if (kind == 0u) {
        let p0x = c.geom_a.x;
        let p0y = c.geom_a.y;
        let dx = c.geom_a.z;
        let dy = c.geom_a.w;
        let len2 = max(c.geom_b.x, 1e-12);
        let feather = clamp(c.geom_b.y, 0.0, 1.0);
        let t = ((u - p0x) * dx + (v - p0y) * dy) / len2;
        let half_f = 0.5 * feather;
        raw = smoothstep_calc(0.5 - half_f, 0.5 + half_f, t);
    } else if (kind == 1u) {
        let cx = c.geom_a.x;
        let cy = c.geom_a.y;
        let inv_rx = c.geom_a.z;
        let inv_ry = c.geom_a.w;
        let feather = clamp(c.geom_b.y, 0.0, 1.0);
        let ddx = (u - cx) * inv_rx;
        let ddy = (v - cy) * inv_ry;
        let d = sqrt(ddx * ddx + ddy * ddy);
        raw = 1.0 - smoothstep_calc(1.0 - max(feather, 1e-3), 1.0, d);
    } else if (kind == 2u) {
        let slot = i32(c.kind_mode_invert_pad.w);
        raw = textureSampleLevel(atlas, samp, vec2<f32>(u, v), slot, 0.0).x;
    } else if (kind == 3u) {
        let luma = 0.2126 * display_rgb.x + 0.7152 * display_rgb.y + 0.0722 * display_rgb.z;
        raw = luma_range_weight(luma, c.geom_a.x, c.geom_a.y, c.geom_a.z);
    } else if (kind == 4u) {
        raw = color_range_weight(display_rgb, c.geom_a.xyz, c.geom_b.x, c.geom_b.y);
    } else if (kind == 5u) {
        raw = polygon_weight(
            u32(c.geom_a.x),
            u32(c.geom_a.y),
            vec2<f32>(u, v),
            c.geom_a.z,
        );
    }
    let inverted = c.kind_mode_invert_pad.z;
    var r = raw;
    if (inverted == 1u) { r = 1.0 - r; }
    return clamp(r, 0.0, 1.0);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.out_size.x || gid.y >= p.out_size.y) { return; }
    let ow = f32(p.out_size.x);
    let oh = f32(p.out_size.y);
    let du = (f32(gid.x) + 0.5) / ow;
    let dv = (f32(gid.y) + 0.5) / oh;
    let scene = display_to_scene(du, dv);
    let u = scene.x;
    let v = scene.y;
    let display_rgb = tone_apply_rgb(textureLoad(display_tex, vec2<i32>(i32(gid.x), i32(gid.y)), 0).rgb);
    var w: f32 = 0.0;
    let n = p.n_components;
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let c = comps[i];
        let cw = component_weight(c, u, v, display_rgb);
        let mode = c.kind_mode_invert_pad.y;
        if (mode == 0u) {
            w = 1.0 - (1.0 - w) * (1.0 - cw);
        } else if (mode == 1u) {
            w = w * (1.0 - cw);
        } else {
            w = w * cw;
        }
    }
    if (p.flags.w == 1u) { w = 1.0 - w; }
    let final_w = clamp(w * p.layer_amount, 0.0, 1.0);
    textureStore(weight_out, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(final_w, 0.0, 0.0, 1.0));
}
"#;

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

#[allow(clippy::too_many_arguments)]
pub fn pack_params(
    out_w: u32,
    out_h: u32,
    n_components: u32,
    layer_amount: f32,
    crop: [f32; 4],
    flags: [u32; 4],
    geom_extra2: [f32; 4],
    geom_extra3: [f32; 4],
    lens: [f32; 4],
    perspective: [f32; 12],
) -> MaskWeightParams {
    MaskWeightParams {
        out_size: [out_w, out_h],
        n_components,
        layer_amount,
        crop,
        flags,
        geom_extra2,
        geom_extra3,
        lens,
        perspective,
    }
}
