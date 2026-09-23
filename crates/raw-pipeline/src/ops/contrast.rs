use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::math::smoothstep;
use crate::vmath;

pub struct ContrastOp;

pub const CONTRAST_GAMMA: f32 = 2.2;
pub const CONTRAST_ROLLOFF_LO: f32 = 1.0;
pub const CONTRAST_ROLLOFF_HI: f32 = 1.01;

pub(crate) fn contrast_strength(amount: f32) -> f32 {
    (amount.clamp(-1.0, 1.0) * 1.25).exp2()
}

#[inline(always)]
pub(crate) fn apply_perceptual_contrast(v: f32, s: f32) -> f32 {
    let p = vmath::pow(v.max(0.0), 1.0 / CONTRAST_GAMMA);
    let pc = p.clamp(0.0, 1.0);
    let low = pc < 0.5;
    let base = if low { 2.0 * pc } else { 2.0 * (1.0 - pc) };
    let half = 0.5 * vmath::pow(base, s);
    let op = if low { half } else { 1.0 - half };
    let lin = vmath::pow(op, CONTRAST_GAMMA);
    let m = smoothstep(CONTRAST_ROLLOFF_LO, CONTRAST_ROLLOFF_HI, v);
    lin * (1.0 - m) + v * m
}

impl Op for ContrastOp {
    fn id(&self) -> &'static str {
        "contrast"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
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
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let s = contrast_strength(edits.basic.contrast as f32 / 100.0);
        Some(CpuFusedOp::Contrast { s })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "contrast",
            include_str!("../../assets/shaders/ops/contrast.wgsl"),
            "lin = contrast_apply(lin, p.contrast);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = contrast_strength(edits.basic.contrast as f32 / 100.0);
    }
}
