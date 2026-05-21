use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
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
        edits.geometry.rotate != 0 || edits.geometry.flip_h || edits.geometry.flip_v
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
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "_geom_noop",
            functions: "",
            apply: "",
            vec4_count: 0,
        })
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.geometry.rotate == 0 && !edits.geometry.flip_h && !edits.geometry.flip_v {
            return None;
        }
        Some(serde_json::json!({
            "rotate": edits.geometry.rotate,
            "flip_h": edits.geometry.flip_h,
            "flip_v": edits.geometry.flip_v,
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
    }
}
