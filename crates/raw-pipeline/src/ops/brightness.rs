use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use crate::math::{luma, smoothstep};

pub struct BrightnessOp;

pub const BRIGHTNESS_K: f32 = 1.5;
pub const BRIGHTNESS_ROLLOFF_LO: f32 = 0.9;
pub const BRIGHTNESS_ROLLOFF_HI: f32 = 1.0;
pub const BRIGHTNESS_MAX_GAIN: f32 = 8.0;

pub(crate) fn apply_brightness_rgb(r: f32, g: f32, b: f32, amount: f32) -> (f32, f32, f32) {
    let y0 = luma(r, g, b);
    if y0 <= 1e-5 {
        return (r, g, b);
    }
    let a = amount.clamp(-1.0, 1.0);
    let yc = y0.clamp(0.0, 1.0);
    let d = yc + (1.0 - yc) * (-a * BRIGHTNESS_K).exp2();
    let yl = if d > 1e-5 { yc / d } else { yc };
    let guard = y0.max(r).max(g).max(b);
    let rolloff = smoothstep(BRIGHTNESS_ROLLOFF_LO, BRIGHTNESS_ROLLOFF_HI, guard);
    let y1 = yl * (1.0 - rolloff) + y0 * rolloff;
    let s = (y1 / y0).clamp(0.0, BRIGHTNESS_MAX_GAIN);
    (r * s, g * s, b * s)
}

impl Op for BrightnessOp {
    fn id(&self) -> &'static str {
        "brightness"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        5
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.brightness != 0.0
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.brightness == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.brightness }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.brightness = v;
        }
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let amount = (edits.basic.brightness as f32 / 100.0).clamp(-1.0, 1.0);
        Some(CpuFusedOp::Brightness { amount })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "brightness",
            include_str!("../../assets/shaders/ops/brightness.wgsl"),
            "lin = brightness_apply(lin, p.brightness);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = (edits.basic.brightness as f32 / 100.0).clamp(-1.0, 1.0);
    }
}
