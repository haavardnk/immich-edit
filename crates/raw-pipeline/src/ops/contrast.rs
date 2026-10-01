use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::math::fast;

pub struct ContrastOp;

pub const CONTRAST_GAMMA: f32 = 2.2;

pub(crate) fn contrast_strength(amount: f32) -> f32 {
    (amount.clamp(-1.0, 1.0) * 1.25).exp2()
}

#[inline(always)]
fn contrast_bias(x: f32, k: f32) -> f32 {
    x / (k * (1.0 - x) + 1.0)
}

#[inline(always)]
pub(crate) fn contrast_curve(v: f32, s: f32) -> f32 {
    if v >= 1.0 {
        return v;
    }
    if v <= 0.0 {
        return v * fast::pow(s, -CONTRAST_GAMMA);
    }
    let p = fast::pow(v, 1.0 / CONTRAST_GAMMA);
    let k = s - 1.0;
    let out = if p < 0.5 {
        0.5 * contrast_bias(2.0 * p, k)
    } else {
        1.0 - 0.5 * contrast_bias(2.0 - 2.0 * p, k)
    };
    fast::pow(out, CONTRAST_GAMMA)
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
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
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
