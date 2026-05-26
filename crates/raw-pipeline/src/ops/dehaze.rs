use super::LinearImage;
use super::{EditOperator, OpContext, OpKind, Stage};
use crate::PipelineResult;
use crate::cpu::dehaze::apply_dehaze;
use crate::edits::Edits;

pub struct DehazeOp;

impl EditOperator for DehazeOp {
    fn id(&self) -> &'static str {
        "dehaze"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn kind(&self) -> OpKind {
        OpKind::Spatial
    }
    fn order(&self) -> i32 {
        -10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.dehaze != 0.0
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let amt = (edits.basic.dehaze as f32 / 100.0).clamp(-1.0, 1.0);
        apply_dehaze(image, amt);
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.dehaze == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.dehaze }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.dehaze = v;
        }
    }
}
