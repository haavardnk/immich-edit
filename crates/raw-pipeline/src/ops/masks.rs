use super::{EditOperator, LinearImage, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, MaskLayer};

pub struct MasksOp;

impl EditOperator for MasksOp {
    fn id(&self) -> &'static str {
        "masks"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        1000
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        false
    }
    fn apply_cpu(
        &self,
        _image: &mut LinearImage,
        _ctx: &OpContext,
        _edits: &Edits,
    ) -> PipelineResult<()> {
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.masks.is_empty() {
            return None;
        }
        Some(serde_json::json!({ "layers": edits.masks }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let Some(arr) = value.get("layers").and_then(|v| v.as_array()) else {
            return;
        };
        let layers: Vec<MaskLayer> = arr
            .iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
        edits.masks = layers;
    }
}
