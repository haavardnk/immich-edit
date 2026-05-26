use super::LinearImage;
use super::{OpContext, OpMeta, ResourceNeed, SpatialOp, Stage};
use crate::PipelineResult;
use crate::cpu::presence::apply_presence;
use crate::edits::Edits;

pub struct ClarityOp;

impl OpMeta for ClarityOp {
    fn id(&self) -> &'static str {
        "clarity"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -20
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.clarity != 0.0
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.clarity == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.clarity }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.clarity = v;
        }
    }
}

impl SpatialOp for ClarityOp {
    fn resource_needs(&self, edits: &Edits) -> Vec<ResourceNeed> {
        if !self.is_active(edits) {
            return Vec::new();
        }
        vec![ResourceNeed::LumaPyramid { max_radius_px: 64 }]
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
                clarity: edits.basic.clarity,
                ..Default::default()
            },
            ..Default::default()
        };
        apply_presence(image, &only);
        Ok(())
    }
}
