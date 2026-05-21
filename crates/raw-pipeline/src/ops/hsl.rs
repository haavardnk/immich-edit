use super::LinearImage;
use super::{EditOperator, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, HSL_BANDS};
use rayon::prelude::*;

pub struct HslOp;

const BAND_CENTERS_DEG: [f32; HSL_BANDS] = [0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 270.0, 300.0];
const BAND_SIGMA_DEG: f32 = 25.0;

fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    let d = max - min;
    if d < 1e-6 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let mut h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h *= 60.0;
    (h, s, l)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

fn hsl_to_rgb(h_deg: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s <= 0.0 {
        return (l, l, l);
    }
    let h = (h_deg.rem_euclid(360.0)) / 360.0;
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

fn hue_dist(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

fn band_weights(h_deg: f32) -> [f32; HSL_BANDS] {
    let mut w = [0.0f32; HSL_BANDS];
    let sigma2 = BAND_SIGMA_DEG * BAND_SIGMA_DEG;
    for i in 0..HSL_BANDS {
        let d = hue_dist(h_deg, BAND_CENTERS_DEG[i]);
        w[i] = (-(d * d) / (2.0 * sigma2)).exp();
    }
    w
}

impl EditOperator for HslOp {
    fn id(&self) -> &'static str {
        "hsl"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        50
    }
    fn is_active(&self, edits: &Edits) -> bool {
        !edits.color.hsl.is_zero()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let bands = edits.color.hsl.bands;
        let hue_shifts: [f32; HSL_BANDS] =
            std::array::from_fn(|i| (bands[i].hue as f32) / 100.0 * 30.0);
        let sat_gains: [f32; HSL_BANDS] = std::array::from_fn(|i| (bands[i].sat as f32) / 100.0);
        let lum_gains: [f32; HSL_BANDS] = std::array::from_fn(|i| (bands[i].lum as f32) / 100.0);

        image.rgb.par_chunks_mut(3).for_each(|p| {
            let r = p[0];
            let g = p[1];
            let b = p[2];
            let (h, s, l) = rgb_to_hsl(r.clamp(0.0, 2.0), g.clamp(0.0, 2.0), b.clamp(0.0, 2.0));
            if s < 1e-4 {
                return;
            }
            let w = band_weights(h);
            let mut hue_delta = 0.0f32;
            let mut sat_delta = 0.0f32;
            let mut lum_delta = 0.0f32;
            for i in 0..HSL_BANDS {
                hue_delta += hue_shifts[i] * w[i];
                sat_delta += sat_gains[i] * w[i];
                lum_delta += lum_gains[i] * w[i];
            }
            let new_h = h + hue_delta;
            let new_s = (s * (1.0 + sat_delta)).clamp(0.0, 1.0);
            let new_l = (l + lum_delta * 0.3).clamp(0.0, 1.0);
            let (nr, ng, nb) = hsl_to_rgb(new_h, new_s, new_l);
            p[0] = nr;
            p[1] = ng;
            p[2] = nb;
        });
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if !self.is_active(edits) {
            return None;
        }
        let arr: Vec<serde_json::Value> = edits
            .color
            .hsl
            .bands
            .iter()
            .map(|b| serde_json::json!({"hue": b.hue, "sat": b.sat, "lum": b.lum}))
            .collect();
        Some(serde_json::json!({ "bands": arr }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let Some(arr) = value.get("bands").and_then(|v| v.as_array()) else {
            return;
        };
        for (i, item) in arr.iter().take(HSL_BANDS).enumerate() {
            let band = &mut edits.color.hsl.bands[i];
            if let Some(v) = item.get("hue").and_then(|v| v.as_f64()) {
                band.hue = v;
            }
            if let Some(v) = item.get("sat").and_then(|v| v.as_f64()) {
                band.sat = v;
            }
            if let Some(v) = item.get("lum").and_then(|v| v.as_f64()) {
                band.lum = v;
            }
        }
    }
}
