use super::{GpuOp, Op, OpContext, Stage};
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;

pub struct SaturationOp;

impl Op for SaturationOp {
    fn id(&self) -> &'static str {
        "saturation"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.saturation != 0.0
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
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let factor = 1.0 + edits.basic.saturation as f32 / 100.0;
        Some(CpuFusedOp::Saturation { factor })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "saturation",
            include_str!("../../assets/shaders/ops/saturation.wgsl"),
            "lin = saturation_apply(lin, p.saturation);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.basic.saturation as f32 / 100.0;
    }
}
