use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct ContrastOp;

impl EditOperator for ContrastOp {
    fn id(&self) -> &'static str {
        "contrast"
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
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let factor = 1.0 + edits.basic.contrast as f32 / 100.0;
        image
            .rgb
            .par_iter_mut()
            .for_each(|v| *v = (*v - 0.5) * factor + 0.5);
        Ok(())
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let factor = 1.0 + edits.basic.contrast as f32 / 100.0;
        Some(CpuFusedOp::Contrast { factor })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "contrast",
            "fn contrast_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { if (p.x == 0.0) { return c; } let f = 1.0 + p.x; return (c - vec3<f32>(0.5)) * f + vec3<f32>(0.5); }",
            "lin = contrast_apply(lin, p.contrast);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.basic.contrast as f32 / 100.0;
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
}
