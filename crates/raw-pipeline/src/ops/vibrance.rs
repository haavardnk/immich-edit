use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::fused::CpuFusedOp;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct VibranceOp;

impl EditOperator for VibranceOp {
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
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let amount = edits.basic.vibrance as f32 / 100.0;
        image.rgb.par_chunks_exact_mut(3).for_each(|px| {
            let r = px[0];
            let g = px[1];
            let b = px[2];
            let max = r.max(g).max(b);
            let min = r.min(g).min(b);
            let sat = (max - min).clamp(0.0, 1.0);
            let factor = 1.0 + amount * (1.0 - sat);
            let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            px[0] = luma + (r - luma) * factor;
            px[1] = luma + (g - luma) * factor;
            px[2] = luma + (b - luma) * factor;
        });
        Ok(())
    }
    fn cpu_fused(&self, edits: &Edits, _ctx: &OpContext) -> Option<CpuFusedOp> {
        let amount = edits.basic.vibrance as f32 / 100.0;
        Some(CpuFusedOp::Vibrance { amount })
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "vibrance",
            "fn vibrance_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { if (p.x == 0.0) { return c; } let mx = max(max(c.r, c.g), c.b); let mn = min(min(c.r, c.g), c.b); let sat = clamp(mx - mn, 0.0, 1.0); let f = 1.0 + p.x * (1.0 - sat); let luma = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b; return vec3<f32>(luma) + (c - vec3<f32>(luma)) * f; }",
            "lin = vibrance_apply(lin, p.vibrance);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.basic.vibrance as f32 / 100.0;
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
}
