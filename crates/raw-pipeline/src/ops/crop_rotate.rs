use rayon::prelude::*;

use super::LinearImage;
use super::{EditOperator, GpuOp, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{AspectLock, CropRect, Edits};
use crate::geom;

pub struct CropRotateOp;

impl EditOperator for CropRotateOp {
    fn id(&self) -> &'static str {
        "crop_rotate"
    }
    fn stage(&self) -> Stage {
        Stage::Geometry
    }
    fn order(&self) -> i32 {
        1
    }
    fn is_active(&self, edits: &Edits) -> bool {
        let crop_active = edits.geometry.crop.map(|c| !c.is_full()).unwrap_or(false);
        edits.geometry.rotate_angle.abs() > 1e-4 || crop_active
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let sw = image.width as f32;
        let sh = image.height as f32;
        let angle = edits.geometry.rotate_angle;
        let crop = edits.geometry.crop.unwrap_or(CropRect::full());
        let bbox = geom::rotated_bbox(sw, sh, angle);
        let bw = bbox.w;
        let bh = bbox.h;
        let out_w = (crop.w * bw).round().max(1.0) as usize;
        let out_h = (crop.h * bh).round().max(1.0) as usize;
        let a = geom::deg_to_rad(angle);
        let cos_a = a.cos();
        let sin_a = a.sin();
        let src = &image.rgb;
        let src_w = image.width;
        let src_h = image.height;
        let mut out = vec![0.0f32; out_w * out_h * 3];
        out.par_chunks_mut(out_w * 3)
            .enumerate()
            .for_each(|(oy, row)| {
                let v = (oy as f32 + 0.5) / out_h as f32;
                for ox in 0..out_w {
                    let u = (ox as f32 + 0.5) / out_w as f32;
                    let bx_rel = crop.x + u * crop.w;
                    let by_rel = crop.y + v * crop.h;
                    let cx = (bx_rel - 0.5) * bw;
                    let cy = (by_rel - 0.5) * bh;
                    let sx = cx * cos_a + cy * sin_a;
                    let sy = -cx * sin_a + cy * cos_a;
                    let fx = sx + sw / 2.0 - 0.5;
                    let fy = sy + sh / 2.0 - 0.5;
                    let d = ox * 3;
                    if fx < 0.0 || fy < 0.0 || fx > (src_w - 1) as f32 || fy > (src_h - 1) as f32 {
                        row[d] = 0.0;
                        row[d + 1] = 0.0;
                        row[d + 2] = 0.0;
                        continue;
                    }
                    let x0 = fx.floor() as usize;
                    let y0 = fy.floor() as usize;
                    let x1 = (x0 + 1).min(src_w - 1);
                    let y1 = (y0 + 1).min(src_h - 1);
                    let tx = fx - x0 as f32;
                    let ty = fy - y0 as f32;
                    let i00 = (y0 * src_w + x0) * 3;
                    let i10 = (y0 * src_w + x1) * 3;
                    let i01 = (y1 * src_w + x0) * 3;
                    let i11 = (y1 * src_w + x1) * 3;
                    for c in 0..3 {
                        let v00 = src[i00 + c];
                        let v10 = src[i10 + c];
                        let v01 = src[i01 + c];
                        let v11 = src[i11 + c];
                        let a0 = v00 + (v10 - v00) * tx;
                        let a1 = v01 + (v11 - v01) * tx;
                        row[d + c] = a0 + (a1 - a0) * ty;
                    }
                }
            });
        image.rgb = out;
        image.width = out_w;
        image.height = out_h;
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "_crop_rotate_noop",
            functions: "",
            apply: "",
            vec4_count: 0,
        })
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let g = &edits.geometry;
        let crop_active = g.crop.map(|c| !c.is_full()).unwrap_or(false);
        let angle_active = g.rotate_angle.abs() > 1e-4;
        let aspect_active = !matches!(g.aspect, AspectLock::Original);
        if !crop_active && !angle_active && !aspect_active {
            return None;
        }
        let mut obj = serde_json::Map::new();
        if angle_active {
            obj.insert("angle".into(), serde_json::json!(g.rotate_angle));
        }
        if let Some(c) = g.crop {
            obj.insert(
                "crop".into(),
                serde_json::json!({ "x": c.x, "y": c.y, "w": c.w, "h": c.h }),
            );
        }
        obj.insert("aspect".into(), serde_json::to_value(g.aspect).ok()?);
        Some(serde_json::Value::Object(obj))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(a) = value.get("angle").and_then(|v| v.as_f64()) {
            edits.geometry.rotate_angle = a as f32;
        }
        if let Some(c) = value.get("crop") {
            let x = c.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let y = c.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
            let w = c.get("w").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            let h = c.get("h").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
            edits.geometry.crop = Some(CropRect { x, y, w, h });
        }
        if let Some(a) = value.get("aspect") {
            if let Ok(v) = serde_json::from_value::<AspectLock>(a.clone()) {
                edits.geometry.aspect = v;
            }
        }
    }
}
