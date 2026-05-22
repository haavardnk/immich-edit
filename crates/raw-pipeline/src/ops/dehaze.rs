use super::LinearImage;
use super::local::{apply_luma_delta, box_blur_separable, luma_buffer};
use super::{EditOperator, OpContext, ResourceNeed, Stage};
use crate::PipelineResult;
use crate::edits::Edits;
use rayon::prelude::*;

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
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let amount = (edits.basic.dehaze as f32 / 100.0).clamp(-1.0, 1.0) * 0.4;
        let radius = (image.width.min(image.height) / 8).max(20);
        let luma = luma_buffer(image);
        let blurred = box_blur_separable(&luma, image.width, image.height, radius);
        let new_luma: Vec<f32> = luma
            .par_iter()
            .zip(blurred.par_iter())
            .map(|(&l, &b)| (l + amount * (l - b)).max(0.0))
            .collect();
        apply_luma_delta(image, &new_luma);
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
