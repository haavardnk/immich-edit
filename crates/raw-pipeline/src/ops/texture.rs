use super::LinearImage;
use super::local::{apply_luma_delta, box_blur_separable, luma_buffer};
use super::{EditOperator, OpContext, ResourceNeed, Stage};
use crate::PipelineResult;
use crate::edits::Edits;
use rayon::prelude::*;

pub struct TextureOp;

impl EditOperator for TextureOp {
    fn id(&self) -> &'static str {
        "texture"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -30
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.texture != 0.0
    }
    fn resource_needs(&self, edits: &Edits) -> Vec<ResourceNeed> {
        if !self.is_active(edits) {
            return Vec::new();
        }
        vec![ResourceNeed::LumaPyramid { max_radius_px: 2 }]
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let amount = (edits.basic.texture as f32 / 100.0).clamp(-1.0, 1.0);
        let luma = luma_buffer(image);
        let blurred = box_blur_separable(&luma, image.width, image.height, 1);
        let new_luma: Vec<f32> = luma
            .par_iter()
            .zip(blurred.par_iter())
            .map(|(&l, &b)| (l + amount * (l - b)).max(0.0))
            .collect();
        apply_luma_delta(image, &new_luma);
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.texture == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.texture }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.texture = v;
        }
    }
}
