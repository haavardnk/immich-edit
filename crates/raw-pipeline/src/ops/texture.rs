use super::LinearImage;
use super::{OpContext, OpMeta, ResourceNeed, SpatialOp, Stage};
use crate::PipelineResult;
use crate::cpu::presence::apply_presence;
use crate::edits::Edits;

pub struct TextureOp;

impl OpMeta for TextureOp {
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

impl SpatialOp for TextureOp {
    fn resource_needs(&self, edits: &Edits) -> Vec<ResourceNeed> {
        if !self.is_active(edits) {
            return Vec::new();
        }
        vec![ResourceNeed::LumaPyramid { max_radius_px: 6 }]
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
                texture: edits.basic.texture,
                ..Default::default()
            },
            ..Default::default()
        };
        apply_presence(image, &only);
        Ok(())
    }
}
