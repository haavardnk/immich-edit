use super::{FusedOp, GpuOp, OpContext, OpMeta, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct BrightnessOp;

const K: f32 = 1.5;

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub(crate) fn apply_brightness_rgb(r: f32, g: f32, b: f32, amount: f32) -> (f32, f32, f32) {
    let y0 = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    if y0 <= 1e-5 {
        return (r, g, b);
    }
    let a = amount.clamp(-1.0, 1.0);
    let yc = y0.clamp(0.0, 1.0);
    let d = yc + (1.0 - yc) * (-a * K).exp2();
    let yl = if d > 1e-5 { yc / d } else { yc };
    let guard = y0.max(r).max(g).max(b);
    let rolloff = smoothstep(0.9, 1.0, guard);
    let y1 = yl * (1.0 - rolloff) + y0 * rolloff;
    let s = (y1 / y0).clamp(0.0, 8.0);
    (r * s, g * s, b * s)
}

impl OpMeta for BrightnessOp {
    fn id(&self) -> &'static str {
        "brightness"
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
}

impl FusedOp for BrightnessOp {
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let amount = (edits.basic.brightness as f32 / 100.0).clamp(-1.0, 1.0);
        Some(CpuFusedOp::Brightness { amount })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "brightness",
            "fn brightness_smoothstep(e0: f32, e1: f32, x: f32) -> f32 { let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0); return t * t * (3.0 - 2.0 * t); } fn brightness_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { let a = clamp(p.x, -1.0, 1.0); let y0 = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z; if (y0 <= 1e-5) { return c; } let yc = clamp(y0, 0.0, 1.0); let d = yc + (1.0 - yc) * exp2(-a * 1.5); var yl: f32; if (d > 1e-5) { yl = yc / d; } else { yl = yc; } let guard = max(y0, max(c.x, max(c.y, c.z))); let rolloff = brightness_smoothstep(0.9, 1.0, guard); let y1 = yl * (1.0 - rolloff) + y0 * rolloff; let s = clamp(y1 / y0, 0.0, 8.0); return c * s; }",
            "lin = brightness_apply(lin, p.brightness);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = (edits.basic.brightness as f32 / 100.0).clamp(-1.0, 1.0);
    }
}
