use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
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
    let sum: f32 = w.iter().sum();
    if sum > 1.0 {
        for v in &mut w {
            *v /= sum;
        }
    }
    w
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl EditOperator for HslOp {
    fn id(&self) -> &'static str {
        "hsl"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        -10
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
            let gate = smoothstep(0.05, 0.20, s);
            let mut hue_delta = 0.0f32;
            let mut sat_delta = 0.0f32;
            let mut lum_delta = 0.0f32;
            for i in 0..HSL_BANDS {
                hue_delta += hue_shifts[i] * w[i];
                sat_delta += sat_gains[i] * w[i];
                lum_delta += lum_gains[i] * w[i];
            }
            hue_delta *= gate;
            sat_delta *= gate;
            lum_delta *= gate;
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
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "hsl",
            functions: HSL_WGSL,
            apply: "lin = hsl_apply(lin);",
            vec4_count: HSL_BANDS,
            kind: crate::ops::GpuOpKind::Normal,
        })
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        if !self.is_active(edits) {
            return None;
        }
        let bands = edits.color.hsl.bands;
        let hue_shifts: [f32; HSL_BANDS] =
            std::array::from_fn(|i| (bands[i].hue as f32) / 100.0 * 30.0);
        let sat_gains: [f32; HSL_BANDS] = std::array::from_fn(|i| (bands[i].sat as f32) / 100.0);
        let lum_gains: [f32; HSL_BANDS] = std::array::from_fn(|i| (bands[i].lum as f32) / 100.0);
        Some(CpuFusedOp::Hsl {
            hue_shifts,
            sat_gains,
            lum_gains,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        for (i, band) in edits.color.hsl.bands.iter().enumerate() {
            dst[i * 4] = band.hue as f32;
            dst[i * 4 + 1] = band.sat as f32;
            dst[i * 4 + 2] = band.lum as f32;
            dst[i * 4 + 3] = 0.0;
        }
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

const HSL_WGSL: &str = r#"
fn hsl_rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
    let mx = max(max(c.r, c.g), c.b);
    let mn = min(min(c.r, c.g), c.b);
    let l = (mx + mn) * 0.5;
    let d = mx - mn;
    if (d < 1e-6) { return vec3<f32>(0.0, 0.0, l); }
    var s: f32;
    if (l > 0.5) { s = d / (2.0 - mx - mn); } else { s = d / (mx + mn); }
    var h: f32;
    if (mx == c.r) {
        var k = (c.g - c.b) / d;
        if (c.g < c.b) { k = k + 6.0; }
        h = k;
    } else if (mx == c.g) {
        h = (c.b - c.r) / d + 2.0;
    } else {
        h = (c.r - c.g) / d + 4.0;
    }
    return vec3<f32>(h * 60.0, s, l);
}
fn hsl_hue_to_rgb(p0: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if (t < 0.0) { t = t + 1.0; }
    if (t > 1.0) { t = t - 1.0; }
    if (t < 1.0 / 6.0) { return p0 + (q - p0) * 6.0 * t; }
    if (t < 0.5) { return q; }
    if (t < 2.0 / 3.0) { return p0 + (q - p0) * (2.0 / 3.0 - t) * 6.0; }
    return p0;
}
fn hsl_hsl_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let h_deg = c.x; let s = c.y; let l = c.z;
    if (s <= 0.0) { return vec3<f32>(l); }
    let h_wrap = h_deg - floor(h_deg / 360.0) * 360.0;
    let h_norm = h_wrap / 360.0;
    var q: f32;
    if (l < 0.5) { q = l * (1.0 + s); } else { q = l + s - l * s; }
    let p0 = 2.0 * l - q;
    return vec3<f32>(
        hsl_hue_to_rgb(p0, q, h_norm + 1.0 / 3.0),
        hsl_hue_to_rgb(p0, q, h_norm),
        hsl_hue_to_rgb(p0, q, h_norm - 1.0 / 3.0)
    );
}
fn hsl_hue_dist(a: f32, b: f32) -> f32 {
    let raw = a - b;
    let wrapped = raw - floor(raw / 360.0) * 360.0;
    return min(wrapped, 360.0 - wrapped);
}
fn hsl_apply(c_in: vec3<f32>) -> vec3<f32> {
    let cc = clamp(c_in, vec3<f32>(0.0), vec3<f32>(2.0));
    let hsl = hsl_rgb_to_hsl(cc);
    if (hsl.y < 1e-4) { return c_in; }
    var centers: array<f32, 8> = array<f32, 8>(0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 270.0, 300.0);
    let sigma2 = 625.0;
    var w: array<f32, 8>;
    var w_sum: f32 = 0.0;
    for (var i: i32 = 0; i < 8; i = i + 1) {
        let d = hsl_hue_dist(hsl.x, centers[i]);
        w[i] = exp(-(d * d) / (2.0 * sigma2));
        w_sum = w_sum + w[i];
    }
    if (w_sum > 1.0) {
        for (var i: i32 = 0; i < 8; i = i + 1) {
            w[i] = w[i] / w_sum;
        }
    }
    let gate = smoothstep(0.05, 0.20, hsl.y);
    var hue_d: f32 = 0.0;
    var sat_d: f32 = 0.0;
    var lum_d: f32 = 0.0;
    for (var i: i32 = 0; i < 8; i = i + 1) {
        hue_d = hue_d + (p.hsl[i].x / 100.0 * 30.0) * w[i];
        sat_d = sat_d + (p.hsl[i].y / 100.0) * w[i];
        lum_d = lum_d + (p.hsl[i].z / 100.0) * w[i];
    }
    hue_d = hue_d * gate;
    sat_d = sat_d * gate;
    lum_d = lum_d * gate;
    let new_h = hsl.x + hue_d;
    let new_s = clamp(hsl.y * (1.0 + sat_d), 0.0, 1.0);
    let new_l = clamp(hsl.z + lum_d * 0.3, 0.0, 1.0);
    return hsl_hsl_to_rgb(vec3<f32>(new_h, new_s, new_l));
}
"#;
