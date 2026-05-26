use crate::cpu::fused::{CpuFusedOp, FusedSegment, apply_one};
use crate::edits::{Edits, MaskComponentKind, MaskComponentMode, MaskLayer};
use crate::mask_raster::{MaskRaster, RasterMap};
use crate::ops::LinearImage;
use crate::ops::lens_distortion::{LensWarpParams, mask_uv_to_scene_uv};
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum ComponentKindEval {
    Linear {
        p0: (f32, f32),
        dir: (f32, f32),
        len2: f32,
        feather: f32,
    },
    Radial {
        center: (f32, f32),
        inv_radius: (f32, f32),
        feather: f32,
    },
    Brush {
        raster_id: String,
        raster: Option<Arc<MaskRaster>>,
    },
}

#[derive(Clone, Debug)]
pub struct ComponentEval {
    pub mode: MaskComponentMode,
    pub opacity: f32,
    pub invert: bool,
    pub kind: ComponentKindEval,
}

#[derive(Clone, Debug)]
pub struct LayerEval {
    pub amount: f32,
    pub components: Vec<ComponentEval>,
}

pub fn build_layer_evals(layers: &[MaskLayer], rasters: &RasterMap) -> Vec<LayerEval> {
    layers
        .iter()
        .filter(|l| l.is_effective())
        .map(|l| build_layer_eval(l, rasters))
        .collect()
}

pub fn build_layer_eval(layer: &MaskLayer, rasters: &RasterMap) -> LayerEval {
    let components: Vec<ComponentEval> = layer
        .components
        .iter()
        .filter(|c| c.enabled && c.opacity.abs() > 1e-6)
        .map(|c| {
            let kind = match &c.kind {
                MaskComponentKind::Linear { p0, p1, feather } => {
                    let dx = p1.x - p0.x;
                    let dy = p1.y - p0.y;
                    let len2 = (dx * dx + dy * dy).max(1e-12);
                    ComponentKindEval::Linear {
                        p0: (p0.x, p0.y),
                        dir: (dx, dy),
                        len2,
                        feather: feather.clamp(0.0, 1.0),
                    }
                }
                MaskComponentKind::Radial {
                    center,
                    radius_xy,
                    feather,
                } => {
                    let ix = if radius_xy.x.abs() < 1e-6 {
                        0.0
                    } else {
                        1.0 / radius_xy.x
                    };
                    let iy = if radius_xy.y.abs() < 1e-6 {
                        0.0
                    } else {
                        1.0 / radius_xy.y
                    };
                    ComponentKindEval::Radial {
                        center: (center.x, center.y),
                        inv_radius: (ix, iy),
                        feather: feather.clamp(0.0, 1.0),
                    }
                }
                MaskComponentKind::Brush { raster_id } => ComponentKindEval::Brush {
                    raster_id: raster_id.clone(),
                    raster: rasters.get(raster_id).cloned(),
                },
            };
            ComponentEval {
                mode: c.mode,
                opacity: c.opacity.clamp(0.0, 1.0),
                invert: c.invert,
                kind,
            }
        })
        .collect();
    LayerEval {
        amount: layer.amount.clamp(0.0, 1.0),
        components,
    }
}

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0).max(1e-6)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline(always)]
fn component_weight(c: &ComponentEval, u: f32, v: f32) -> f32 {
    let raw = match &c.kind {
        ComponentKindEval::Linear {
            p0,
            dir,
            len2,
            feather,
        } => {
            let t = ((u - p0.0) * dir.0 + (v - p0.1) * dir.1) / *len2;
            let half = 0.5 * feather.clamp(0.0, 1.0);
            smoothstep(0.5 - half, 0.5 + half, t)
        }
        ComponentKindEval::Radial {
            center,
            inv_radius,
            feather,
        } => {
            let dx = (u - center.0) * inv_radius.0;
            let dy = (v - center.1) * inv_radius.1;
            let d = (dx * dx + dy * dy).sqrt();
            1.0 - smoothstep(1.0 - feather.max(1e-3), 1.0, d)
        }
        ComponentKindEval::Brush { raster, .. } => match raster {
            Some(r) => r.sample_bilinear(u, v),
            None => 0.0,
        },
    };
    let r = if c.invert { 1.0 - raw } else { raw };
    (r * c.opacity).clamp(0.0, 1.0)
}

#[inline(always)]
pub fn fold_layer_weight(layer: &LayerEval, u: f32, v: f32) -> f32 {
    let mut w: f32 = 0.0;
    for c in &layer.components {
        let cw = component_weight(c, u, v);
        w = match c.mode {
            MaskComponentMode::Add => 1.0 - (1.0 - w) * (1.0 - cw),
            MaskComponentMode::Subtract => w * (1.0 - cw),
            MaskComponentMode::Intersect => w * cw,
        };
    }
    (w * layer.amount).clamp(0.0, 1.0)
}

pub fn apply_segment_masked(
    image: &mut LinearImage,
    base_segment: &FusedSegment,
    layer_segments: &[FusedSegment],
    layers: &[LayerEval],
    lens_warp: &LensWarpParams,
) {
    if base_segment.is_empty() && layer_segments.iter().all(|s| s.is_empty()) {
        return;
    }
    let w = image.width;
    let h = image.height;
    let inv_w = 1.0 / w.max(1) as f32;
    let inv_h = 1.0 / h.max(1) as f32;
    let row_floats = w * 3;
    let base_ops = base_segment.ops.as_slice();
    let layer_ops: Vec<&[CpuFusedOp]> = layer_segments.iter().map(|s| s.ops.as_slice()).collect();
    let warp_active = !lens_warp.is_identity();

    image
        .rgb
        .par_chunks_exact_mut(row_floats)
        .enumerate()
        .for_each(|(y, row)| {
            let row_base = y * w;
            let v = (y as f32 + 0.5) * inv_h;
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let i = row_base + x;
                let u = (x as f32 + 0.5) * inv_w;
                let (su, sv) = if warp_active {
                    let s = mask_uv_to_scene_uv(lens_warp, [u, v]);
                    (s[0], s[1])
                } else {
                    (u, v)
                };
                let r0 = px[0];
                let g0 = px[1];
                let b0 = px[2];
                let mut br = r0;
                let mut bg = g0;
                let mut bb = b0;
                for op in base_ops {
                    apply_one(op, i, &mut br, &mut bg, &mut bb);
                }
                let mut out_r = br;
                let mut out_g = bg;
                let mut out_b = bb;
                for (li, layer) in layers.iter().enumerate() {
                    let lw = fold_layer_weight(layer, su, sv);
                    if lw <= 1e-4 {
                        continue;
                    }
                    let mut lr = r0;
                    let mut lg = g0;
                    let mut lb = b0;
                    for op in layer_ops[li] {
                        apply_one(op, i, &mut lr, &mut lg, &mut lb);
                    }
                    out_r = out_r + (lr - out_r) * lw;
                    out_g = out_g + (lg - out_g) * lw;
                    out_b = out_b + (lb - out_b) * lw;
                }
                px[0] = out_r;
                px[1] = out_g;
                px[2] = out_b;
            }
        });
}

pub fn render_mask_weight(image: &mut LinearImage, layer: &LayerEval, lens_warp: &LensWarpParams) {
    let w = image.width;
    let h = image.height;
    let inv_w = 1.0 / w.max(1) as f32;
    let inv_h = 1.0 / h.max(1) as f32;
    let row_floats = w * 3;
    let warp_active = !lens_warp.is_identity();
    image
        .rgb
        .par_chunks_exact_mut(row_floats)
        .enumerate()
        .for_each(|(y, row)| {
            let v = (y as f32 + 0.5) * inv_h;
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let u = (x as f32 + 0.5) * inv_w;
                let (su, sv) = if warp_active {
                    let s = mask_uv_to_scene_uv(lens_warp, [u, v]);
                    (s[0], s[1])
                } else {
                    (u, v)
                };
                let lw = fold_layer_weight(layer, su, sv);
                px[0] = lw;
                px[1] = lw;
                px[2] = lw;
            }
        });
}

pub fn effective_edits_for_layer(global: &Edits, layer: &MaskLayer) -> Edits {
    let mut out = global.clone();
    let d = &layer.edits;
    if let Some(v) = d.exposure_ev {
        out.basic.exposure_ev = (out.basic.exposure_ev + v).clamp(-5.0, 5.0);
    }
    if let Some(v) = d.brightness {
        out.basic.brightness = (out.basic.brightness + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.contrast {
        out.basic.contrast = (out.basic.contrast + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.saturation {
        out.basic.saturation = (out.basic.saturation + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.vibrance {
        out.basic.vibrance = (out.basic.vibrance + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.wb_temp {
        out.basic.wb_temp = (out.basic.wb_temp + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.wb_tint {
        out.basic.wb_tint = (out.basic.wb_tint + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.highlights {
        out.tone.highlights = (out.tone.highlights + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.shadows {
        out.tone.shadows = (out.tone.shadows + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.whites {
        out.tone.whites = (out.tone.whites + v).clamp(-100.0, 100.0);
    }
    if let Some(v) = d.blacks {
        out.tone.blacks = (out.tone.blacks + v).clamp(-100.0, 100.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::{LensEdits, MaskComponent, MaskSource, Vec2f};

    fn linear(id: &str, p0: Vec2f, p1: Vec2f, feather: f32) -> MaskComponent {
        MaskComponent {
            id: id.into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            opacity: 1.0,
            invert: false,
            kind: MaskComponentKind::Linear { p0, p1, feather },
            source: MaskSource::Manual,
        }
    }

    #[test]
    fn linear_gradient_weights() {
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![linear(
                "c",
                Vec2f { x: 0.0, y: 0.0 },
                Vec2f { x: 1.0, y: 0.0 },
                1.0,
            )],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
        let w_left = fold_layer_weight(&eval, 0.0, 0.5);
        let w_right = fold_layer_weight(&eval, 1.0, 0.5);
        let w_mid = fold_layer_weight(&eval, 0.5, 0.5);
        if w_left > 0.05 {
            panic!("expected 0 at p0, got {w_left}");
        }
        if w_right < 0.95 {
            panic!("expected 1 at p1, got {w_right}");
        }
        if (w_mid - 0.5).abs() > 0.1 {
            panic!("expected ~0.5 at mid, got {w_mid}");
        }
    }

    #[test]
    fn radial_inside_outside() {
        let mut comp = linear("c", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 0.0, y: 0.0 }, 0.0);
        comp.kind = MaskComponentKind::Radial {
            center: Vec2f { x: 0.5, y: 0.5 },
            radius_xy: Vec2f { x: 0.2, y: 0.2 },
            feather: 0.1,
        };
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![comp],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
        let inside = fold_layer_weight(&eval, 0.5, 0.5);
        let outside = fold_layer_weight(&eval, 0.9, 0.9);
        if inside < 0.95 {
            panic!("expected ~1 inside, got {inside}");
        }
        if outside > 0.05 {
            panic!("expected ~0 outside, got {outside}");
        }
    }

    #[test]
    fn subtract_carves_out() {
        let add = linear("a", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 1.0, y: 0.0 }, 0.0);
        let mut sub = linear("s", Vec2f { x: 0.0, y: 0.0 }, Vec2f { x: 1.0, y: 0.0 }, 0.0);
        sub.mode = MaskComponentMode::Subtract;
        sub.kind = MaskComponentKind::Radial {
            center: Vec2f { x: 0.5, y: 0.5 },
            radius_xy: Vec2f { x: 0.1, y: 0.1 },
            feather: 0.05,
        };
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![add, sub],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
        let on_carve = fold_layer_weight(&eval, 0.5, 0.5);
        let right_clear = fold_layer_weight(&eval, 0.95, 0.5);
        if on_carve > 0.1 {
            panic!("expected near-zero where subtracted, got {on_carve}");
        }
        if right_clear < 0.85 {
            panic!("expected ~1 outside carve, got {right_clear}");
        }
    }

    #[test]
    fn masked_exposure_brightens_only_right_half() {
        let w = 8;
        let h = 4;
        let rgb = vec![0.5f32; w * h * 3];
        let mut image = LinearImage::new(rgb, w, h);
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![linear(
                "c",
                Vec2f { x: 0.0, y: 0.0 },
                Vec2f { x: 1.0, y: 0.0 },
                0.0,
            )],
            edits: crate::edits::MaskedEdits {
                exposure_ev: Some(2.0),
                ..Default::default()
            },
        };
        let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
        let base = FusedSegment::default();
        let mut layer_seg = FusedSegment::default();
        layer_seg.push(CpuFusedOp::Exposure { factor: 4.0 });
        let warp = LensWarpParams::from_edits(&Default::default(), w as u32, h as u32);
        apply_segment_masked(&mut image, &base, &[layer_seg], &[eval], &warp);
        let left = image.rgb[0];
        let right = image.rgb[3 * (w - 1)];
        if (left - 0.5).abs() > 1e-3 {
            panic!("expected left untouched, got {left}");
        }
        if (right - 2.0).abs() > 1e-3 {
            panic!("expected right ~2.0 (4x base), got {right}");
        }
    }

    #[test]
    fn render_mask_weight_writes_grayscale_gradient() {
        let w = 16;
        let h = 4;
        let mut image = LinearImage::new(vec![0.0f32; w * h * 3], w, h);
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![linear(
                "c",
                Vec2f { x: 0.0, y: 0.0 },
                Vec2f { x: 1.0, y: 0.0 },
                1.0,
            )],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
        let warp = LensWarpParams::from_edits(&Default::default(), w as u32, h as u32);
        render_mask_weight(&mut image, &eval, &warp);
        let left = image.rgb[0];
        let right = image.rgb[3 * (w - 1)];
        if left > 0.1 {
            panic!("expected ~0 on left, got {left}");
        }
        if right < 0.9 {
            panic!("expected ~1 on right, got {right}");
        }
        if (image.rgb[0] - image.rgb[1]).abs() > 1e-6 || (image.rgb[0] - image.rgb[2]).abs() > 1e-6
        {
            panic!("expected grayscale (r=g=b)");
        }
    }

    #[test]
    fn brush_raster_samples_bilinear() {
        let bytes = vec![0u8, 0, 255, 255];
        let raster = Arc::new(MaskRaster::new(2, 2, bytes).unwrap());
        let mut rasters = RasterMap::new();
        rasters.insert("r1".into(), raster);
        let comp = MaskComponent {
            id: "c".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            opacity: 1.0,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "r1".into(),
            },
            source: MaskSource::Manual,
        };
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![comp],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &rasters);
        let tl = fold_layer_weight(&eval, 0.0, 0.0);
        let br = fold_layer_weight(&eval, 1.0, 1.0);
        let mid = fold_layer_weight(&eval, 0.5, 0.5);
        if tl > 0.05 {
            panic!("expected ~0 at (0,0), got {tl}");
        }
        if br < 0.95 {
            panic!("expected ~1 at (1,1), got {br}");
        }
        if (mid - 0.5).abs() > 0.1 {
            panic!("expected ~0.5 at center, got {mid}");
        }
    }

    #[test]
    fn brush_missing_raster_yields_zero_weight() {
        let comp = MaskComponent {
            id: "c".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            opacity: 1.0,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "missing".into(),
            },
            source: MaskSource::Manual,
        };
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![comp],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &RasterMap::new());
        let w = fold_layer_weight(&eval, 0.5, 0.5);
        if w > 1e-6 {
            panic!("expected 0 with missing raster, got {w}");
        }
    }

    #[test]
    fn effective_edits_adds_and_clamps_brightness() {
        let mut g = Edits::default();
        g.basic.brightness = 80.0;
        let mut layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![],
            edits: Default::default(),
        };
        layer.edits.brightness = Some(50.0);
        let eff = effective_edits_for_layer(&g, &layer);
        if (eff.basic.brightness - 100.0).abs() > 1e-6 {
            panic!("expected clamp to 100, got {}", eff.basic.brightness);
        }
        layer.edits.brightness = Some(-200.0);
        let eff = effective_edits_for_layer(&g, &layer);
        if (eff.basic.brightness - (-100.0)).abs() > 1e-6 {
            panic!("expected clamp to -100, got {}", eff.basic.brightness);
        }
    }

    #[test]
    fn scene_space_mask_anchors_through_lens_warp() {
        let w = 64usize;
        let h = 32usize;
        let layer = MaskLayer {
            id: "l".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
            amount: 1.0,
            components: vec![linear(
                "c",
                Vec2f { x: 0.0, y: 0.0 },
                Vec2f { x: 1.0, y: 0.0 },
                0.0,
            )],
            edits: Default::default(),
        };
        let eval = build_layer_eval(&layer, &crate::mask_raster::empty_rasters());
        let identity = LensWarpParams::from_edits(&LensEdits::default(), w as u32, h as u32);
        let lens = LensEdits {
            profile_enabled: true,
            k1: -0.15,
            constrain_crop: true,
            ..Default::default()
        };
        let warp = LensWarpParams::from_edits(&lens, w as u32, h as u32);
        if warp.is_identity() {
            panic!("warp should be non-identity for anchoring test");
        }
        let mut img_warp = LinearImage::new(vec![0.0f32; w * h * 3], w, h);
        render_mask_weight(&mut img_warp, &eval, &warp);
        let mut img_id = LinearImage::new(vec![0.0f32; w * h * 3], w, h);
        render_mask_weight(&mut img_id, &eval, &identity);
        let samples = [(0.5, 0.5), (0.7, 0.5), (0.5, 0.3), (0.8, 0.7)];
        for (u, v) in samples {
            let mx = ((u * w as f32) as usize).min(w - 1);
            let my = ((v * h as f32) as usize).min(h - 1);
            let warped = img_warp.rgb[(my * w + mx) * 3];
            let scene = mask_uv_to_scene_uv(&warp, [u, v]);
            let sx = ((scene[0] * w as f32) as usize).min(w - 1);
            let sy = ((scene[1] * h as f32) as usize).min(h - 1);
            let identity_at_scene = img_id.rgb[(sy * w + sx) * 3];
            if (warped - identity_at_scene).abs() > 0.05 {
                panic!(
                    "anchor mismatch at ({u},{v}): warped={warped} identity_at_scene={identity_at_scene}"
                );
            }
        }
    }
}
