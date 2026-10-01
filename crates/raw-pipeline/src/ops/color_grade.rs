use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::{ColorGradeRegion, Edits};
use crate::math::linear_srgb_to_oklab;

pub struct ColorGradeOp;

pub const COLOR_GRADE_CHROMA: f32 = 0.4;
pub const COLOR_GRADE_LUM_EV: f32 = 1.5;
pub const COLOR_GRADE_PIVOT_BASE: f32 = 0.5;
pub const COLOR_GRADE_PIVOT_RANGE: f32 = 0.3;
pub const COLOR_GRADE_FEATHER_BASE: f32 = 0.15;
pub const COLOR_GRADE_FEATHER_RANGE: f32 = 0.25;

pub(crate) fn hue_dir(hue_deg: f32) -> [f32; 3] {
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

fn region_offset(region: &ColorGradeRegion) -> [f32; 3] {
    let [_, a, b] = linear_srgb_to_oklab(hue_dir(region.hue as f32).map(|c| c + 0.5));
    let gain = COLOR_GRADE_CHROMA * region.sat as f32 / 100.0 / a.hypot(b);
    [
        a * gain,
        b * gain,
        COLOR_GRADE_LUM_EV * region.lum as f32 / 100.0,
    ]
}

fn regions(edits: &Edits) -> [[f32; 3]; 4] {
    let cg = &edits.color.color_grade;
    [&cg.shadows, &cg.midtones, &cg.highlights, &cg.global].map(region_offset)
}

impl Op for ColorGradeOp {
    fn id(&self) -> &'static str {
        "color_grade"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
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
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
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
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "color_grade",
            functions: include_str!("../../assets/shaders/ops/color_grade.wgsl"),
            apply: "lin = color_grade_apply(lin);",
            vec4_count: 5,
        })
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        if !self.is_active(edits) {
            return None;
        }
        let cg = &edits.color.color_grade;
        Some(CpuFusedOp::ColorGrade {
            regions: regions(edits),
            balance: (cg.balance as f32) / 100.0,
            blend: (cg.blend as f32) / 100.0,
        })
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        let cg = &edits.color.color_grade;
        for (i, r) in regions(edits).iter().enumerate() {
            dst[i * 4..i * 4 + 3].copy_from_slice(r);
            dst[i * 4 + 3] = 0.0;
        }
        dst[16] = (cg.balance as f32) / 100.0;
        dst[17] = (cg.blend as f32) / 100.0;
        dst[18] = 0.0;
        dst[19] = 0.0;
    }
}
