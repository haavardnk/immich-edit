use super::LinearImage;
use super::{EditOperator, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, EffectsEdits};
use rayon::prelude::*;

pub struct GrainOp;

impl EditOperator for GrainOp {
    fn id(&self) -> &'static str {
        "grain"
    }
    fn stage(&self) -> Stage {
        Stage::Output
    }
    fn order(&self) -> i32 {
        2
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.effects.grain_active()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_grain(image, &edits.effects);
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let e = &edits.effects;
        if !e.grain_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": e.grain_amount,
            "size": e.grain_size,
            "roughness": e.grain_roughness,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let e: &mut EffectsEdits = &mut edits.effects;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            e.grain_amount = v;
        }
        if let Some(v) = value.get("size").and_then(|v| v.as_f64()) {
            e.grain_size = v;
        }
        if let Some(v) = value.get("roughness").and_then(|v| v.as_f64()) {
            e.grain_roughness = v;
        }
    }
}

const Y_R: f32 = 0.2126;
const Y_G: f32 = 0.7152;
const Y_B: f32 = 0.0722;

#[inline]
fn pcg_hash(mut x: u32) -> u32 {
    x = x.wrapping_mul(747796405).wrapping_add(2891336453);
    let word = ((x >> ((x >> 28).wrapping_add(4))) ^ x).wrapping_mul(277803737);
    (word >> 22) ^ word
}

#[inline]
fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let v = pcg_hash((x as u32).wrapping_mul(0x27d4eb2d) ^ pcg_hash((y as u32) ^ seed));
    (v as f32) / (u32::MAX as f32)
}

#[inline]
fn fade(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - xi as f32;
    let yf = y - yi as f32;
    let u = fade(xf);
    let v = fade(yf);
    let a = hash2(xi, yi, seed);
    let b = hash2(xi + 1, yi, seed);
    let c = hash2(xi, yi + 1, seed);
    let d = hash2(xi + 1, yi + 1, seed);
    lerp(lerp(a, b, u), lerp(c, d, u), v)
}

pub fn apply_grain(image: &mut LinearImage, e: &EffectsEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let amount = (e.grain_amount / 100.0) as f32;
    let size = (e.grain_size / 100.0) as f32;
    let roughness = (e.grain_roughness / 100.0) as f32;
    let cell = lerp(1.0, 8.0, size);
    let fine_cell = (cell * 0.5).max(1.0);
    let seed = (w as u32) ^ (h as u32).rotate_left(13);
    let seed_fine = seed ^ 0x9E3779B9;
    let strength = amount * 0.15;

    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let yf = y as f32;
            for x in 0..w {
                let xf = x as f32;
                let base = value_noise(xf / cell, yf / cell, seed);
                let fine = value_noise(xf / fine_cell, yf / fine_cell, seed_fine);
                let n = lerp(base, fine, roughness) * 2.0 - 1.0;
                let delta = n * strength;
                let i = x * 3;
                let r = row[i];
                let g = row[i + 1];
                let b = row[i + 2];
                let yv = Y_R * r + Y_G * g + Y_B * b;
                let scale = if yv > 1e-6 { (yv + delta) / yv } else { 1.0 };
                row[i] = (r * scale).clamp(0.0, 4.0);
                row[i + 1] = (g * scale).clamp(0.0, 4.0);
                row[i + 2] = (b * scale).clamp(0.0, 4.0);
            }
        });
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

    fn mean_luma(img: &LinearImage) -> f32 {
        let mut acc = 0.0_f64;
        let n = img.width * img.height;
        for i in 0..n {
            let r = img.rgb[i * 3] as f64;
            let g = img.rgb[i * 3 + 1] as f64;
            let b = img.rgb[i * 3 + 2] as f64;
            acc += Y_R as f64 * r + Y_G as f64 * g + Y_B as f64 * b;
        }
        (acc / n as f64) as f32
    }

    fn variance(img: &LinearImage) -> f32 {
        let m = mean_luma(img) as f64;
        let mut acc = 0.0_f64;
        let n = img.width * img.height;
        for i in 0..n {
            let r = img.rgb[i * 3] as f64;
            let g = img.rgb[i * 3 + 1] as f64;
            let b = img.rgb[i * 3 + 2] as f64;
            let y = Y_R as f64 * r + Y_G as f64 * g + Y_B as f64 * b;
            acc += (y - m).powi(2);
        }
        (acc / n as f64) as f32
    }

    #[test]
    fn amount_zero_identity() {
        let mut img = make_image(32, 32, 0.5);
        let orig = img.rgb.clone();
        apply_grain(&mut img, &defaults());
        if img.rgb != orig {
            panic!("grain at 0 should be identity");
        }
    }

    #[test]
    fn variance_increases_on_flat_patch() {
        let flat = make_image(64, 64, 0.5);
        let mut grainy = make_image(64, 64, 0.5);
        let mut e = defaults();
        e.grain_amount = 80.0;
        apply_grain(&mut grainy, &e);
        let v0 = variance(&flat);
        let v1 = variance(&grainy);
        if v1 <= v0 + 1e-4 {
            panic!("variance {v1} should exceed flat {v0}");
        }
    }

    #[test]
    fn deterministic_two_runs() {
        let mut a = make_image(32, 32, 0.5);
        let mut b = make_image(32, 32, 0.5);
        let mut e = defaults();
        e.grain_amount = 50.0;
        apply_grain(&mut a, &e);
        apply_grain(&mut b, &e);
        if a.rgb != b.rgb {
            panic!("grain should be deterministic");
        }
    }

    #[test]
    fn mean_luma_stays_within_one_percent() {
        let mut img = make_image(128, 128, 0.5);
        let before = mean_luma(&img);
        let mut e = defaults();
        e.grain_amount = 60.0;
        apply_grain(&mut img, &e);
        let after = mean_luma(&img);
        let drift = (after - before).abs() / before;
        if drift > 0.01 {
            panic!("mean drifted {drift} (before {before}, after {after})");
        }
    }

    #[test]
    fn size_changes_spatial_frequency() {
        fn neighbor_corr(img: &LinearImage) -> f32 {
            let m = mean_luma(img);
            let mut num = 0.0_f64;
            let mut den = 0.0_f64;
            for y in 0..img.height {
                for x in 0..img.width - 1 {
                    let a = img.rgb[(y * img.width + x) * 3] - m;
                    let b = img.rgb[(y * img.width + x + 1) * 3] - m;
                    num += (a * b) as f64;
                    den += (a * a) as f64;
                }
            }
            if den < 1e-9 { 0.0 } else { (num / den) as f32 }
        }
        let mut small = make_image(128, 128, 0.5);
        let mut big = make_image(128, 128, 0.5);
        let mut es = defaults();
        es.grain_amount = 60.0;
        es.grain_size = 0.0;
        let mut eb = defaults();
        eb.grain_amount = 60.0;
        eb.grain_size = 100.0;
        apply_grain(&mut small, &es);
        apply_grain(&mut big, &eb);
        let cs = neighbor_corr(&small);
        let cb = neighbor_corr(&big);
        if cb <= cs {
            panic!("larger grain {cb} should have higher neighbor correlation than small {cs}");
        }
    }
}
