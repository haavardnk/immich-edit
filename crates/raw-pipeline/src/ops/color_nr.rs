use super::LinearImage;
use super::{EditOperator, GpuOpKind, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{DetailEdits, Edits};

pub struct ColorNrOp;

impl EditOperator for ColorNrOp {
    fn id(&self) -> &'static str {
        "color_nr"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -40
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.color_nr_active()
    }
    fn gpu_kind(&self) -> GpuOpKind {
        GpuOpKind::Detail
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
        let d = &edits.detail;
        if !d.color_nr_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": d.color_nr_amount,
            "detail": d.color_nr_detail,
            "smoothness": d.color_nr_smoothness,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let d: &mut DetailEdits = &mut edits.detail;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            d.color_nr_amount = v;
        }
        if let Some(v) = value.get("detail").and_then(|v| v.as_f64()) {
            d.color_nr_detail = v;
        }
        if let Some(v) = value.get("smoothness").and_then(|v| v.as_f64()) {
            d.color_nr_smoothness = v;
        }
    }
}
