use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct ExposureOp;

impl EditOperator for ExposureOp {
    fn id(&self) -> &'static str {
        "exposure"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        0
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.exposure_ev != 0.0
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let factor = 2.0f32.powf(edits.basic.exposure_ev as f32);
        Some(CpuFusedOp::Exposure { factor })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "exposure",
            "fn exposure_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { return c * p.x; }",
            "lin = exposure_apply(lin, p.exposure);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = 2.0f32.powf(edits.basic.exposure_ev as f32);
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.exposure_ev == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "ev": edits.basic.exposure_ev }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("ev").and_then(|v| v.as_f64()) {
            edits.basic.exposure_ev = v;
        }
    }
}
