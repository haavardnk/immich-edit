use super::{Edits, LinearImage, Op, OpContext, Stage};
use crate::PipelineResult;

pub struct Lut3dOp;

impl Op for Lut3dOp {
    fn id(&self) -> &'static str {
        "lut_3d"
    }
    fn stage(&self) -> Stage {
        Stage::Color
    }
    fn order(&self) -> i32 {
        200
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        false
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let lut = &edits.color.lut_3d;
        let id = lut.lut_id.as_ref().filter(|s| !s.is_empty())?;
        let mut obj = serde_json::Map::new();
        obj.insert("lut_id".into(), id.clone().into());
        obj.insert("amount".into(), lut.amount.into());
        Some(serde_json::Value::Object(obj))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(id) = value.get("lut_id").and_then(|v| v.as_str())
            && !id.is_empty()
        {
            edits.color.lut_3d.lut_id = Some(id.to_string());
        }
        if let Some(amount) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.color.lut_3d.amount = amount;
        }
    }
    fn apply_cpu(
        &self,
        _image: &mut LinearImage,
        _ctx: &OpContext,
        _edits: &Edits,
    ) -> PipelineResult<()> {
        Ok(())
    }
}
