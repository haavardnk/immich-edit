use super::{EditOperator, GpuOp, LinearImage, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct BrightnessOp;

impl EditOperator for BrightnessOp {
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
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let offset = edits.basic.brightness as f32 / 100.0;
        image.rgb.par_iter_mut().for_each(|v| *v += offset);
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp::new(
            "brightness",
            "fn brightness_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> { return c + vec3<f32>(p.x); }",
            "lin = brightness_apply(lin, p.brightness);",
        ))
    }
    fn write_gpu_uniform(&self, edits: &Edits, _ctx: &OpContext, dst: &mut [f32]) {
        dst[0] = edits.basic.brightness as f32 / 100.0;
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
