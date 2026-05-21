use super::LinearImage;
use super::{EditOperator, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::transform;
use crate::edits::Edits;

pub struct GeometryOp;

impl EditOperator for GeometryOp {
    fn id(&self) -> &'static str {
        "geometry"
    }
    fn stage(&self) -> Stage {
        Stage::Geometry
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.rotate != 0 || edits.flip_h || edits.flip_v || edits.crop.is_some()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let mut steps = edits.rotate / 90;
        while steps > 0 {
            let (rotated, nw, nh) = transform::rotate_90(&image.rgb, image.width, image.height);
            image.rgb = rotated;
            image.width = nw;
            image.height = nh;
            steps -= 1;
        }
        if edits.flip_h {
            transform::flip_horizontal(&mut image.rgb, image.width, image.height);
        }
        if edits.flip_v {
            transform::flip_vertical(&mut image.rgb, image.width, image.height);
        }
        if let Some(crop) = &edits.crop {
            let (cropped, cw, ch) = transform::crop(
                &image.rgb,
                image.width,
                image.height,
                crop.x,
                crop.y,
                crop.width,
                crop.height,
            );
            image.rgb = cropped;
            image.width = cw;
            image.height = ch;
        }
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.rotate == 0 && !edits.flip_h && !edits.flip_v && edits.crop.is_none() {
            return None;
        }
        Some(serde_json::json!({
            "rotate": edits.rotate,
            "flip_h": edits.flip_h,
            "flip_v": edits.flip_v,
            "crop": edits.crop,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("rotate").and_then(|v| v.as_u64()) {
            edits.rotate = v as u16;
        }
        if let Some(v) = value.get("flip_h").and_then(|v| v.as_bool()) {
            edits.flip_h = v;
        }
        if let Some(v) = value.get("flip_v").and_then(|v| v.as_bool()) {
            edits.flip_v = v;
        }
        if let Some(c) = value.get("crop") {
            if c.is_null() {
                edits.crop = None;
            } else if let Ok(parsed) = serde_json::from_value::<crate::edits::CropRect>(c.clone()) {
                edits.crop = Some(parsed);
            }
        }
    }
}
