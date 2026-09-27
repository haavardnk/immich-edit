use super::LinearImage;
use super::{Op, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::dehaze::apply_dehaze;
use crate::edits::Edits;

pub struct DehazeOp;

impl Op for DehazeOp {
    fn id(&self) -> &'static str {
        "dehaze"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Pass(super::GpuPass::Dehaze)
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -35
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.basic.dehaze != 0.0
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.basic.dehaze == 0.0 {
            return None;
        }
        Some(serde_json::json!({ "amount": edits.basic.dehaze }))
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            edits.basic.dehaze = v;
        }
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DehazeGrid {
    pub scale: u32,
    pub patch: u32,
    pub guided: u32,
}

impl DehazeGrid {
    pub fn for_dims((w, h): (u32, u32)) -> Self {
        let min_dim = w.min(h);
        let half_min = (min_dim / 2).max(1);
        let patch_full = (min_dim / 200).max(8).min(half_min);
        let guided_full = (min_dim / 50).max(16).min(half_min);
        let scale = if min_dim >= 512 { 4 } else { 1 };
        Self {
            scale,
            patch: (patch_full / scale).max(2),
            guided: (guided_full / scale).max(4),
        }
    }

    pub fn support(&self) -> u32 {
        self.scale * (self.patch + 2 * self.guided + 2)
    }
}
