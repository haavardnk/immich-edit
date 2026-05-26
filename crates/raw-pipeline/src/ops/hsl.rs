use super::{FusedOp, GpuOp, OpContext, OpMeta, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{Edits, HSL_BANDS};

pub struct HslOp;

impl OpMeta for HslOp {
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

impl FusedOp for HslOp {
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
