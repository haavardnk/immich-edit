use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::math::{hue_dist, smoothstep};

pub struct VibranceOp;

#[inline(always)]
pub(crate) fn apply_vibrance_rgb(r: f32, g: f32, b: f32, amount: f32) -> (f32, f32, f32) {
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    let chroma = d.clamp(0.0, 1.0);
    let hue = if d < 1e-6 {
        0.0
    } else if mx == r {
        ((g - b) / d + if g < b { 6.0 } else { 0.0 }) * 60.0
    } else if mx == g {
        ((b - r) / d + 2.0) * 60.0
    } else {
        ((r - g) / d + 4.0) * 60.0
    };
    let effective = if amount > 0.0 {
        let base = amount * 3.0 * (1.0 - smoothstep(0.4, 0.9, chroma));
        let mut skin = 1.0 - smoothstep(10.0, 35.0, hue_dist(hue, 25.0));
        skin *= smoothstep(0.05, 0.20, chroma);
        base * (1.0 + (0.6 - 1.0) * skin)
    } else {
        amount * (1.0 - smoothstep(0.2, 0.8, chroma))
    };
    if effective.abs() < 1e-5 {
        return (r, g, b);
    }
    let factor = 1.0 + effective;
    let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    (
        luma + (r - luma) * factor,
        luma + (g - luma) * factor,
        luma + (b - luma) * factor,
    )
}

impl Op for VibranceOp {
    fn id(&self) -> &'static str {
        "vibrance"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.vibrance != 0.0
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.vibrance == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.vibrance }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.vibrance = v;
        }
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let amount = edits.basic.vibrance as f32 / 100.0;
        Some(CpuFusedOp::Vibrance { amount })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "vibrance",
            include_str!("../../assets/shaders/ops/vibrance.wgsl"),
            "lin = vibrance_apply(lin, p.vibrance.x);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.basic.vibrance as f32 / 100.0;
    }
}
