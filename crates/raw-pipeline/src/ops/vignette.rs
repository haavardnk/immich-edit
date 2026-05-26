use super::LinearImage;
use super::{OpContext, OpMeta, SpatialOp, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, EffectsEdits};
use rayon::prelude::*;

pub struct VignetteOp;

impl OpMeta for VignetteOp {
    fn id(&self) -> &'static str {
        "vignette"
    }
    fn stage(&self) -> Stage {
        Stage::Output
    }
    fn order(&self) -> i32 {
        1
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.effects.vignette_active()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let e = &edits.effects;
        if !e.vignette_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": e.vignette_amount,
            "midpoint": e.vignette_midpoint,
            "feather": e.vignette_feather,
            "roundness": e.vignette_roundness,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let e: &mut EffectsEdits = &mut edits.effects;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            e.vignette_amount = v;
        }
        if let Some(v) = value.get("midpoint").and_then(|v| v.as_f64()) {
            e.vignette_midpoint = v;
        }
        if let Some(v) = value.get("feather").and_then(|v| v.as_f64()) {
            e.vignette_feather = v;
        }
        if let Some(v) = value.get("roundness").and_then(|v| v.as_f64()) {
            e.vignette_roundness = v;
        }
    }
}

impl SpatialOp for VignetteOp {
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_vignette(image, &edits.effects);
        Ok(())
    }
}

pub fn apply_vignette(image: &mut LinearImage, e: &EffectsEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let amount = (e.vignette_amount / 100.0) as f32;
    let midpoint = (e.vignette_midpoint / 100.0) as f32;
    let feather = (e.vignette_feather / 100.0) as f32;
    let roundness = ((e.vignette_roundness + 100.0) / 200.0) as f32;
    let inner = lerp(0.10, 0.90, midpoint);
    let band = lerp(0.02, (1.25 - inner).max(0.02), feather);
    let aspect = w as f32 / h as f32;
    let inv_w = 1.0 / w as f32;
    let inv_h = 1.0 / h as f32;

    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let v = ((y as f32 + 0.5) * inv_h - 0.5) * 2.0;
            for x in 0..w {
                let u = ((x as f32 + 0.5) * inv_w - 0.5) * 2.0;
                let (cx, cy) = if aspect >= 1.0 {
                    (u * aspect, v)
                } else {
                    (u, v / aspect)
                };
                let qx = lerp(u, cx, roundness);
                let qy = lerp(v, cy, roundness);
                let d = (qx * qx + qy * qy).sqrt();
                let t = smoothstep(inner, inner + band, d);
                let gain = (1.0 + amount * t).clamp(0.0, 2.0);
                let i = x * 3;
                row[i] = (row[i] * gain).clamp(0.0, 4.0);
                row[i + 1] = (row[i + 1] * gain).clamp(0.0, 4.0);
                row[i + 2] = (row[i + 2] * gain).clamp(0.0, 4.0);
            }
        });
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[inline]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge1 <= edge0 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: usize, h: usize, val: f32) -> LinearImage {
        LinearImage::new(vec![val; w * h * 3], w, h)
    }

    fn defaults() -> EffectsEdits {
        EffectsEdits::default()
    }

    fn pixel(img: &LinearImage, x: usize, y: usize) -> f32 {
        img.rgb[(y * img.width + x) * 3]
    }

    #[test]
    fn amount_zero_identity() {
        let mut img = make_image(16, 16, 0.5);
        let orig = img.rgb.clone();
        apply_vignette(&mut img, &defaults());
        assert_eq!(img.rgb, orig);
    }

    #[test]
    fn negative_darkens_corners_more_than_center() {
        let mut img = make_image(32, 32, 0.5);
        let mut e = defaults();
        e.vignette_amount = -80.0;
        apply_vignette(&mut img, &e);
        let center = pixel(&img, 16, 16);
        let corner = pixel(&img, 0, 0);
        if corner >= center {
            panic!("corner {corner} should be darker than center {center}");
        }
    }

    #[test]
    fn positive_brightens_corners_more_than_center() {
        let mut img = make_image(32, 32, 0.5);
        let mut e = defaults();
        e.vignette_amount = 80.0;
        apply_vignette(&mut img, &e);
        let center = pixel(&img, 16, 16);
        let corner = pixel(&img, 0, 0);
        if corner <= center {
            panic!("corner {corner} should be brighter than center {center}");
        }
    }

    #[test]
    fn higher_midpoint_protects_center_more() {
        let mut a = make_image(32, 32, 0.5);
        let mut b = make_image(32, 32, 0.5);
        let mut ea = defaults();
        ea.vignette_amount = -80.0;
        ea.vignette_midpoint = 10.0;
        let mut eb = defaults();
        eb.vignette_amount = -80.0;
        eb.vignette_midpoint = 90.0;
        apply_vignette(&mut a, &ea);
        apply_vignette(&mut b, &eb);
        let high = pixel(&b, 16, 4);
        let low = pixel(&a, 16, 4);
        if high <= low {
            panic!("higher midpoint {high} should exceed lower {low}");
        }
    }

    #[test]
    fn roundness_changes_side_falloff_on_wide_image() {
        let mut square = make_image(64, 16, 0.5);
        let mut circle = make_image(64, 16, 0.5);
        let mut es = defaults();
        es.vignette_amount = -80.0;
        es.vignette_roundness = -100.0;
        let mut ec = defaults();
        ec.vignette_amount = -80.0;
        ec.vignette_roundness = 100.0;
        apply_vignette(&mut square, &es);
        apply_vignette(&mut circle, &ec);
        let side_sq = pixel(&square, 40, 8);
        let side_ci = pixel(&circle, 40, 8);
        if side_ci >= side_sq {
            panic!("circular sides {side_ci} should darken more than square {side_sq}");
        }
    }
}
