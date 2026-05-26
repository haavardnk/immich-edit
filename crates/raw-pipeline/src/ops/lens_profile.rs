use super::LinearImage;
use super::{EditOperator, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::Edits;

pub struct LensProfileOp;

impl EditOperator for LensProfileOp {
    fn id(&self) -> &'static str {
        "lens_profile"
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn order(&self) -> i32 {
        -10
    }
    fn is_active(&self, edits: &Edits) -> bool {
        let l = &edits.lens;
        l.profile_enabled
            || l.ca_enabled
            || l.constrain_crop
            || l.k1 != 0.0
            || l.k2 != 0.0
            || l.k3 != 0.0
            || l.vk1 != 0.0
            || l.vk2 != 0.0
            || l.vk3 != 0.0
            || l.ca_red_scale_x10000 != 0.0
            || l.ca_blue_scale_x10000 != 0.0
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
        if !self.is_active(edits) {
            return None;
        }
        let l = &edits.lens;
        Some(serde_json::json!({
            "profile_enabled": l.profile_enabled,
            "ca_enabled": l.ca_enabled,
            "constrain_crop": l.constrain_crop,
            "distortion_amount": l.distortion_amount,
            "vignette_amount": l.vignette_amount,
            "k1": l.k1,
            "k2": l.k2,
            "k3": l.k3,
            "vk1": l.vk1,
            "vk2": l.vk2,
            "vk3": l.vk3,
            "ca_red": l.ca_red_scale_x10000,
            "ca_blue": l.ca_blue_scale_x10000,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let l = &mut edits.lens;
        if let Some(v) = value.get("profile_enabled").and_then(|v| v.as_bool()) {
            l.profile_enabled = v;
        }
        if let Some(v) = value.get("ca_enabled").and_then(|v| v.as_bool()) {
            l.ca_enabled = v;
        }
        if let Some(v) = value.get("constrain_crop").and_then(|v| v.as_bool()) {
            l.constrain_crop = v;
        }
        if let Some(v) = value.get("distortion_amount").and_then(|v| v.as_f64()) {
            l.distortion_amount = v;
        }
        if let Some(v) = value.get("vignette_amount").and_then(|v| v.as_f64()) {
            l.vignette_amount = v;
        }
        if let Some(v) = value.get("k1").and_then(|v| v.as_f64()) {
            l.k1 = v;
        }
        if let Some(v) = value.get("k2").and_then(|v| v.as_f64()) {
            l.k2 = v;
        }
        if let Some(v) = value.get("k3").and_then(|v| v.as_f64()) {
            l.k3 = v;
        }
        if let Some(v) = value.get("vk1").and_then(|v| v.as_f64()) {
            l.vk1 = v;
        }
        if let Some(v) = value.get("vk2").and_then(|v| v.as_f64()) {
            l.vk2 = v;
        }
        if let Some(v) = value.get("vk3").and_then(|v| v.as_f64()) {
            l.vk3 = v;
        }
        if let Some(v) = value.get("ca_red").and_then(|v| v.as_f64()) {
            l.ca_red_scale_x10000 = v;
        }
        if let Some(v) = value.get("ca_blue").and_then(|v| v.as_f64()) {
            l.ca_blue_scale_x10000 = v;
        }
    }
}
