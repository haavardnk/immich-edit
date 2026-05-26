use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct SaturationOp;

impl EditOperator for SaturationOp {
    fn id(&self) -> &'static str {
        "saturation"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.saturation != 0.0
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let factor = 1.0 + edits.basic.saturation as f32 / 100.0;
        Some(CpuFusedOp::Saturation { factor })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "saturation",
            "fn saturation_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { if (p.x == 0.0) { return c; } let f = 1.0 + p.x; let luma = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b; return vec3<f32>(luma) + (c - vec3<f32>(luma)) * f; }",
            "lin = saturation_apply(lin, p.saturation);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.basic.saturation as f32 / 100.0;
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.saturation == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.saturation }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.saturation = v;
        }
    }
}
