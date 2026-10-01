use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::math::{hue_dist, linear_srgb_to_oklab, oklab_to_linear_srgb, smoothstep};

pub struct VibranceOp;

pub const VIBRANCE_GAIN: f32 = 3.0;
pub const VIBRANCE_SAT_LO: f32 = 0.07;
pub const VIBRANCE_SAT_HI: f32 = 0.23;
pub const VIBRANCE_SKIN_HUE_DEG: f32 = 55.0;
pub const VIBRANCE_SKIN_SPREAD_LO_DEG: f32 = 20.0;
pub const VIBRANCE_SKIN_SPREAD_HI_DEG: f32 = 50.0;
pub const VIBRANCE_SKIN_SAT_LO: f32 = 0.01;
pub const VIBRANCE_SKIN_SAT_HI: f32 = 0.04;
pub const VIBRANCE_SKIN_FACTOR: f32 = 0.6;
pub const VIBRANCE_DESAT_LO: f32 = 0.03;
pub const VIBRANCE_DESAT_HI: f32 = 0.18;

#[inline(always)]
pub(crate) fn apply_vibrance_rgb(r: f32, g: f32, b: f32, amount: f32) -> (f32, f32, f32) {
    let [l, a, bb] = linear_srgb_to_oklab([r, g, b]);
    let chroma = a.hypot(bb);
    if l <= 1e-6 || chroma < 1e-6 {
        return (r, g, b);
    }
    let sat = chroma / l;
    let effective = if amount > 0.0 {
        let base =
            amount * VIBRANCE_GAIN * (1.0 - smoothstep(VIBRANCE_SAT_LO, VIBRANCE_SAT_HI, sat));
        let mut skin = 1.0
            - smoothstep(
                VIBRANCE_SKIN_SPREAD_LO_DEG,
                VIBRANCE_SKIN_SPREAD_HI_DEG,
                hue_dist(bb.atan2(a).to_degrees(), VIBRANCE_SKIN_HUE_DEG),
            );
        skin *= smoothstep(VIBRANCE_SKIN_SAT_LO, VIBRANCE_SKIN_SAT_HI, sat);
        base * (1.0 + (VIBRANCE_SKIN_FACTOR - 1.0) * skin)
    } else {
        amount * (1.0 - smoothstep(VIBRANCE_DESAT_LO, VIBRANCE_DESAT_HI, sat))
    };
    if effective.abs() < 1e-5 {
        return (r, g, b);
    }
    let factor = 1.0 + effective;
    let [nr, ng, nb] = oklab_to_linear_srgb([l, a * factor, bb * factor]);
    (nr, ng, nb)
}

impl Op for VibranceOp {
    fn id(&self) -> &'static str {
        "vibrance"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
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
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
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
