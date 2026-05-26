use super::{LinearImage, OpContext, OpMeta, OutputStageOp, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, TonemapKind};

pub struct OutputOp;

impl OpMeta for OutputOp {
    fn id(&self) -> &'static str {
        "output"
    }
    fn stage(&self) -> Stage {
        Stage::Output
    }
    fn order(&self) -> i32 {
        100
    }
    fn is_active(&self, _edits: &Edits) -> bool {
        false
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.output.is_default() {
            return None;
        }
        let mut obj = serde_json::Map::new();
        if edits.output.tonemap != TonemapKind::Default {
            obj.insert(
                "tonemap".into(),
                tonemap_to_str(edits.output.tonemap).into(),
            );
        }
        Some(serde_json::Value::Object(obj))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(s) = value.get("tonemap").and_then(|v| v.as_str()) {
            edits.output.tonemap = tonemap_from_str(s);
        }
    }
}

impl OutputStageOp for OutputOp {
    fn apply_cpu(
        &self,
        _image: &mut LinearImage,
        _ctx: &OpContext,
        _edits: &Edits,
    ) -> PipelineResult<()> {
        Ok(())
    }
}

fn tonemap_to_str(t: TonemapKind) -> &'static str {
    match t {
        TonemapKind::Default => "default",
        TonemapKind::Agx => "agx",
    }
}

fn tonemap_from_str(s: &str) -> TonemapKind {
    match s {
        "agx" => TonemapKind::Agx,
        _ => TonemapKind::Default,
    }
}
