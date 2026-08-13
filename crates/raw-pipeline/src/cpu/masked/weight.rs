use super::{ComponentEval, ComponentKindEval, LayerEval};
use crate::edits::MaskComponentMode;
use crate::math::{luma, smoothstep, srgb_to_linear};

#[inline(always)]
pub(super) fn display_srgb_to_oklab(rgb: [f32; 3]) -> [f32; 3] {
    let r = srgb_to_linear(rgb[0]);
    let g = srgb_to_linear(rgb[1]);
    let b = srgb_to_linear(rgb[2]);
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

#[inline(always)]
fn point_segment_distance(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    let len2 = dx * dx + dy * dy;
    let t = if len2 <= 1e-12 {
        0.0
    } else {
        (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
    };
    let cx = ax + t * dx;
    let cy = ay + t * dy;
    ((px - cx).powi(2) + (py - cy).powi(2)).sqrt()
}

fn polygon_weight(points: &[(f32, f32)], u: f32, v: f32, feather: f32) -> f32 {
    if points.len() < 3 {
        return 0.0;
    }
    let mut inside = false;
    let mut nearest = f32::MAX;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        if (yi > v) != (yj > v) {
            let t = (v - yi) / (yj - yi);
            if u < xi + t * (xj - xi) {
                inside = !inside;
            }
        }
        nearest = nearest.min(point_segment_distance(u, v, xi, yi, xj, yj));
        j = i;
    }
    if !inside {
        return 0.0;
    }
    if feather <= 1e-6 {
        return 1.0;
    }
    smoothstep(0.0, feather, nearest)
}

#[inline(always)]
fn luma_range_weight(luma: f32, min: f32, max: f32, softness: f32) -> f32 {
    if softness <= 1e-6 {
        return if luma >= min && luma <= max { 1.0 } else { 0.0 };
    }
    let lower = smoothstep(min - softness, min, luma);
    let upper = 1.0 - smoothstep(max, max + softness, luma);
    lower * upper
}

#[inline(always)]
fn color_range_weight(rgb: [f32; 3], sample_lab: [f32; 3], tolerance: f32, softness: f32) -> f32 {
    let lab = display_srgb_to_oklab(rgb);
    let dl = lab[0] - sample_lab[0];
    let da = lab[1] - sample_lab[1];
    let db = lab[2] - sample_lab[2];
    let distance = (dl * dl + da * da + db * db).sqrt();
    if softness <= 1e-6 {
        return if distance <= tolerance { 1.0 } else { 0.0 };
    }
    1.0 - smoothstep(tolerance, tolerance + softness, distance)
}

#[inline(always)]
fn component_weight(c: &ComponentEval, u: f32, v: f32, display_rgb: [f32; 3]) -> f32 {
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
        ComponentKindEval::LumaRange { min, max, softness } => {
            let luma = luma(display_rgb[0], display_rgb[1], display_rgb[2]);
            luma_range_weight(luma, *min, *max, *softness)
        }
        ComponentKindEval::ColorRange {
            sample_rgb: _,
            sample_lab,
            tolerance,
            softness,
        } => color_range_weight(display_rgb, *sample_lab, *tolerance, *softness),
        ComponentKindEval::Polygon { points, feather } => polygon_weight(points, u, v, *feather),
    };
    let r = if c.invert { 1.0 - raw } else { raw };
    r.clamp(0.0, 1.0)
}

#[inline(always)]
pub fn fold_layer_weight(layer: &LayerEval, u: f32, v: f32) -> f32 {
    fold_layer_weight_with_display(layer, u, v, [0.0, 0.0, 0.0])
}

#[inline(always)]
pub fn fold_layer_weight_with_display(
    layer: &LayerEval,
    u: f32,
    v: f32,
    display_rgb: [f32; 3],
) -> f32 {
    let mut w: f32 = 0.0;
    for c in &layer.components {
        let cw = component_weight(c, u, v, display_rgb);
        w = match c.mode {
            MaskComponentMode::Add => 1.0 - (1.0 - w) * (1.0 - cw),
            MaskComponentMode::Subtract => w * (1.0 - cw),
            MaskComponentMode::Intersect => w * cw,
        };
    }
    if layer.invert {
        w = 1.0 - w;
    }
    (w * layer.amount).clamp(0.0, 1.0)
}
