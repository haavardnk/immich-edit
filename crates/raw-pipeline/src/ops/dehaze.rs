use super::LinearImage;
use super::{EditOperator, OpContext, ResourceNeed, Stage};
use crate::PipelineResult;
use crate::cpu::presence::apply_presence;
use crate::edits::Edits;

pub struct DehazeOp;

impl EditOperator for DehazeOp {
    fn id(&self) -> &'static str {
        "dehaze"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.dehaze != 0.0
    }
    fn resource_needs(&self, edits: &Edits) -> Vec<ResourceNeed> {
        if !self.is_active(edits) {
            return Vec::new();
        }
        vec![ResourceNeed::LumaPyramid { max_radius_px: 256 }]
    }
    fn gpu_kind(&self) -> super::GpuOpKind {
        super::GpuOpKind::Presence
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let only = Edits {
            basic: crate::edits::BasicEdits {
                dehaze: edits.basic.dehaze,
                ..Default::default()
            },
            ..Default::default()
        };
        apply_presence(image, &only);
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
