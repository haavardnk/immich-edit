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
        edits.geometry.rotate != 0
            || edits.geometry.flip_h
            || edits.geometry.flip_v
            || edits.geometry.crop.is_some()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let mut steps = edits.geometry.rotate / 90;
        while steps > 0 {
            let (rotated, nw, nh) = transform::rotate_90(&image.rgb, image.width, image.height);
            image.rgb = rotated;
            image.width = nw;
            image.height = nh;
            steps -= 1;
        }
        if edits.geometry.flip_h {
            transform::flip_horizontal(&mut image.rgb, image.width, image.height);
        }
        if edits.geometry.flip_v {
            transform::flip_vertical(&mut image.rgb, image.width, image.height);
        }
        if let Some(crop) = &edits.geometry.crop {
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
        if edits.geometry.rotate == 0
            && !edits.geometry.flip_h
            && !edits.geometry.flip_v
            && edits.geometry.crop.is_none()
        {
            return None;
        }
        Some(serde_json::json!({
            "rotate": edits.geometry.rotate,
            "flip_h": edits.geometry.flip_h,
            "flip_v": edits.geometry.flip_v,
            "crop": edits.geometry.crop,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("rotate").and_then(|v| v.as_u64()) {
            edits.geometry.rotate = v as u16;
        }
        if let Some(v) = value.get("flip_h").and_then(|v| v.as_bool()) {
            edits.geometry.flip_h = v;
        }
        if let Some(v) = value.get("flip_v").and_then(|v| v.as_bool()) {
            edits.geometry.flip_v = v;
        }
        if let Some(c) = value.get("crop") {
            if c.is_null() {
                edits.geometry.crop = None;
            } else if let Ok(parsed) = serde_json::from_value::<crate::edits::CropRect>(c.clone()) {
                edits.geometry.crop = Some(parsed);
            }
        }
    }
}
