use super::{GpuPass, GpuRoute, Op, Stage};
use crate::edits::Edits;

pub struct AiDenoiseOp;

impl Op for AiDenoiseOp {
    fn id(&self) -> &'static str {
        "ai_denoise"
    }
    fn gpu_route(&self) -> GpuRoute {
        GpuRoute::Pass(GpuPass::Denoise)
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn order(&self) -> i32 {
        -20
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.ai_denoise_active()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let d = &edits.detail;
        d.ai_denoise.then(|| {
            serde_json::json!({
                "strength": d.ai_denoise_strength,
                "detail": d.ai_denoise_detail,
            })
        })
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let d = &mut edits.detail;
        d.ai_denoise = true;
        if let Some(v) = value.get("strength").and_then(|v| v.as_f64()) {
            d.ai_denoise_strength = v;
        }
        if let Some(v) = value.get("detail").and_then(|v| v.as_f64()) {
            d.ai_denoise_detail = v;
        }
    }
}
