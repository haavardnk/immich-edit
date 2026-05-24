use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{ColorGradeRegion, Edits};
use rayon::prelude::*;

pub struct ColorGradeOp;

fn hue_dir(hue_deg: f32) -> [f32; 3] {
    let h = (hue_deg.rem_euclid(360.0)) / 60.0;
    let x = 1.0 - (h.rem_euclid(2.0) - 1.0).abs();
    let (r, g, b) = match h as i32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    [r - 0.5, g - 0.5, b - 0.5]
}

fn region_offset(region: &ColorGradeRegion) -> ([f32; 3], f32) {
    let s = (region.sat as f32) / 100.0;
    let l = (region.lum as f32) / 100.0;
    let dir = hue_dir(region.hue as f32);
    ([dir[0] * s, dir[1] * s, dir[2] * s], l)
}

fn weights(y: f32, balance: f32, blend: f32) -> (f32, f32, f32) {
    let pivot = 0.5 + 0.3 * balance;
    let feather = 0.15 + 0.25 * blend;
    let s_hi = (pivot + feather * 0.5).clamp(0.001, 0.999);
    let s_lo = (pivot - feather - feather * 0.5).clamp(0.0, s_hi - 0.001);
    let h_lo = (pivot - feather * 0.5).clamp(0.001, 0.999);
    let h_hi = (pivot + feather + feather * 0.5).clamp(h_lo + 0.001, 1.0);
    let shadow = 1.0 - smoothstep(s_lo, s_hi, y);
    let highlight = smoothstep(h_lo, h_hi, y);
    let mid = (1.0 - shadow - highlight).max(0.0);
    (shadow, mid, highlight)
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl EditOperator for ColorGradeOp {
    fn id(&self) -> &'static str {
        "color_grade"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        20
    }
    fn is_active(&self, edits: &Edits) -> bool {
        !edits.color.color_grade.is_zero()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let cg = &edits.color.color_grade;
        let (s_off, s_lum) = region_offset(&cg.shadows);
        let (m_off, m_lum) = region_offset(&cg.midtones);
        let (h_off, h_lum) = region_offset(&cg.highlights);
        let (g_off, g_lum) = region_offset(&cg.global);
        let balance = (cg.balance as f32) / 100.0;
        let blend = (cg.blend as f32) / 100.0;
        let strength = 0.5;

        image.rgb.par_chunks_exact_mut(3).for_each(|px| {
            let r = px[0];
            let g = px[1];
            let b = px[2];
            let y = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 1.0);
            let (ws, wm, wh) = weights(y, balance, blend);
            let or = (ws * s_off[0] + wm * m_off[0] + wh * h_off[0] + g_off[0]) * strength;
            let og = (ws * s_off[1] + wm * m_off[1] + wh * h_off[1] + g_off[1]) * strength;
            let ob = (ws * s_off[2] + wm * m_off[2] + wh * h_off[2] + g_off[2]) * strength;
            let lum = (ws * s_lum + wm * m_lum + wh * h_lum + g_lum) * strength;
            px[0] = (r + or + lum).max(0.0);
            px[1] = (g + og + lum).max(0.0);
            px[2] = (b + ob + lum).max(0.0);
        });
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "color_grade",
            functions: COLOR_GRADE_WGSL,
            apply: "lin = color_grade_apply(lin);",
            vec4_count: 5,
            kind: crate::ops::GpuOpKind::Normal,
        })
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        if !self.is_active(edits) {
            return None;
        }
        let cg = &edits.color.color_grade;
        let (s_off, s_lum) = region_offset(&cg.shadows);
        let (m_off, m_lum) = region_offset(&cg.midtones);
        let (h_off, h_lum) = region_offset(&cg.highlights);
        let (g_off, g_lum) = region_offset(&cg.global);
        Some(CpuFusedOp::ColorGrade {
            s_off,
            s_lum,
            m_off,
            m_lum,
            h_off,
            h_lum,
            g_off,
            g_lum,
            balance: (cg.balance as f32) / 100.0,
            blend: (cg.blend as f32) / 100.0,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        let cg = &edits.color.color_grade;
        let regions = [&cg.shadows, &cg.midtones, &cg.highlights, &cg.global];
        for (i, r) in regions.iter().enumerate() {
            dst[i * 4] = r.hue as f32;
            dst[i * 4 + 1] = (r.sat as f32) / 100.0;
            dst[i * 4 + 2] = (r.lum as f32) / 100.0;
            dst[i * 4 + 3] = 0.0;
        }
        dst[16] = (cg.balance as f32) / 100.0;
        dst[17] = (cg.blend as f32) / 100.0;
        dst[18] = 0.0;
        dst[19] = 0.0;
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if !self.is_active(edits) {
            return None;
        }
        let cg = &edits.color.color_grade;
        let r = |reg: &ColorGradeRegion| serde_json::json!({"hue": reg.hue, "sat": reg.sat, "lum": reg.lum});
        Some(serde_json::json!({
            "shadows": r(&cg.shadows),
            "midtones": r(&cg.midtones),
            "highlights": r(&cg.highlights),
            "global": r(&cg.global),
            "balance": cg.balance,
            "blend": cg.blend,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let cg = &mut edits.color.color_grade;
        let read = |v: &serde_json::Value, out: &mut ColorGradeRegion| {
            if let Some(x) = v.get("hue").and_then(|v| v.as_f64()) {
                out.hue = x;
            }
            if let Some(x) = v.get("sat").and_then(|v| v.as_f64()) {
                out.sat = x;
            }
            if let Some(x) = v.get("lum").and_then(|v| v.as_f64()) {
                out.lum = x;
            }
        };
        if let Some(v) = value.get("shadows") {
            read(v, &mut cg.shadows);
        }
        if let Some(v) = value.get("midtones") {
            read(v, &mut cg.midtones);
        }
        if let Some(v) = value.get("highlights") {
            read(v, &mut cg.highlights);
        }
        if let Some(v) = value.get("global") {
            read(v, &mut cg.global);
        }
        if let Some(x) = value.get("balance").and_then(|v| v.as_f64()) {
            cg.balance = x;
        }
        if let Some(x) = value.get("blend").and_then(|v| v.as_f64()) {
            cg.blend = x;
        }
    }
}

const COLOR_GRADE_WGSL: &str = r#"
fn cg_hue_dir(hue_deg: f32) -> vec3<f32> {
    let h = (hue_deg - floor(hue_deg / 360.0) * 360.0) / 60.0;
    let x = 1.0 - abs((h - floor(h / 2.0) * 2.0) - 1.0);
    let i = i32(floor(h));
    var rgb: vec3<f32>;
    if (i == 0) { rgb = vec3<f32>(1.0, x, 0.0); }
    else if (i == 1) { rgb = vec3<f32>(x, 1.0, 0.0); }
    else if (i == 2) { rgb = vec3<f32>(0.0, 1.0, x); }
    else if (i == 3) { rgb = vec3<f32>(0.0, x, 1.0); }
    else if (i == 4) { rgb = vec3<f32>(x, 0.0, 1.0); }
    else { rgb = vec3<f32>(1.0, 0.0, x); }
    return rgb - vec3<f32>(0.5);
}
fn cg_smooth(e0: f32, e1: f32, x: f32) -> f32 {
    let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
fn cg_region_offset(reg: vec4<f32>) -> vec4<f32> {
    let dir = cg_hue_dir(reg.x) * reg.y;
    return vec4<f32>(dir.x, dir.y, dir.z, reg.z);
}
fn color_grade_apply(c: vec3<f32>) -> vec3<f32> {
    let s = cg_region_offset(p.color_grade[0]);
    let m = cg_region_offset(p.color_grade[1]);
    let h = cg_region_offset(p.color_grade[2]);
    let g = cg_region_offset(p.color_grade[3]);
    let balance = p.color_grade[4].x;
    let blend = p.color_grade[4].y;
    let y = clamp(0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b, 0.0, 1.0);
    let pivot = 0.5 + 0.3 * balance;
    let feather = 0.15 + 0.25 * blend;
    let s_hi = clamp(pivot + feather * 0.5, 0.001, 0.999);
    let s_lo = clamp(pivot - feather - feather * 0.5, 0.0, s_hi - 0.001);
    let h_lo = clamp(pivot - feather * 0.5, 0.001, 0.999);
    let h_hi = clamp(pivot + feather + feather * 0.5, h_lo + 0.001, 1.0);
    let ws = 1.0 - cg_smooth(s_lo, s_hi, y);
    let wh = cg_smooth(h_lo, h_hi, y);
    let wm = max(1.0 - ws - wh, 0.0);
    let strength = 0.5;
    let off = (vec3<f32>(s.x, s.y, s.z) * ws + vec3<f32>(m.x, m.y, m.z) * wm + vec3<f32>(h.x, h.y, h.z) * wh + vec3<f32>(g.x, g.y, g.z)) * strength;
    let lum = (s.w * ws + m.w * wm + h.w * wh + g.w) * strength;
    return max(c + off + vec3<f32>(lum), vec3<f32>(0.0));
}
"#;
