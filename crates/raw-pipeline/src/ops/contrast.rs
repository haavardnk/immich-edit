use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct ContrastOp;

pub(crate) fn contrast_strength(amount: f32) -> f32 {
    (amount.clamp(-1.0, 1.0) * 1.25).exp2()
}

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn apply_perceptual_contrast(v: f32, s: f32) -> f32 {
    let p = v.max(0.0).powf(1.0 / 2.2);
    let pc = p.clamp(0.0, 1.0);
    let op = if pc < 0.5 {
        0.5 * (2.0 * pc).powf(s)
    } else {
        1.0 - 0.5 * (2.0 * (1.0 - pc)).powf(s)
    };
    let lin = op.powf(2.2);
    let m = smoothstep(1.0, 1.01, v);
    lin * (1.0 - m) + v * m
}

impl EditOperator for ContrastOp {
    fn id(&self) -> &'static str {
        "contrast"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        20
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.contrast != 0.0
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let s = contrast_strength(edits.basic.contrast as f32 / 100.0);
        Some(CpuFusedOp::Contrast { s })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "contrast",
            "fn contrast_smoothstep(e0: f32, e1: f32, x: f32) -> f32 { let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0); return t * t * (3.0 - 2.0 * t); } fn contrast_one(v: f32, s: f32) -> f32 { let p = pow(max(v, 0.0), 1.0 / 2.2); let pc = clamp(p, 0.0, 1.0); var op_v: f32; if (pc < 0.5) { op_v = 0.5 * pow(2.0 * pc, s); } else { op_v = 1.0 - 0.5 * pow(2.0 * (1.0 - pc), s); } let lin = pow(op_v, 2.2); let m = contrast_smoothstep(1.0, 1.01, v); return lin * (1.0 - m) + v * m; } fn contrast_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { if (p.x == 1.0) { return c; } return vec3<f32>(contrast_one(c.x, p.x), contrast_one(c.y, p.x), contrast_one(c.z, p.x)); }",
            "lin = contrast_apply(lin, p.contrast);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = contrast_strength(edits.basic.contrast as f32 / 100.0);
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.contrast == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.contrast }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.contrast = v;
        }
    }
}
