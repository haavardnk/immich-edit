use rayon::prelude::*;

use super::LinearImage;
use super::{GpuOp, Op, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::transform;
use crate::edits::{AspectLock, CropRect, Edits};
use crate::geom;
use crate::perspective::PerspectiveEdits;

pub struct TransformOp;

fn catmull_rom_weights(f: f32) -> [f32; 4] {
    let f2 = f * f;
    let f3 = f2 * f;
    [
        -0.5 * f3 + f2 - 0.5 * f,
        1.5 * f3 - 2.5 * f2 + 1.0,
        -1.5 * f3 + 2.0 * f2 + 0.5 * f,
        0.5 * f3 - 0.5 * f2,
    ]
}

impl Op for TransformOp {
    fn id(&self) -> &'static str {
        "transform"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Fused
    }
    fn stage(&self) -> Stage {
        Stage::Geometry
    }
    fn is_active(&self, edits: &Edits) -> bool {
        let g = &edits.geometry;
        let crop_active = g.crop.map(|c| !c.is_full()).unwrap_or(false);
        let persp_active = g.perspective.map(|p| !p.is_identity()).unwrap_or(false);
        g.rotate != 0
            || g.flip_h
            || g.flip_v
            || g.rotate_angle.abs() > 1e-4
            || crop_active
            || persp_active
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let g = &edits.geometry;
        let crop_active = g.crop.map(|c| !c.is_full()).unwrap_or(false);
        let angle_active = g.rotate_angle.abs() > 1e-4;
        let aspect_active = !matches!(g.aspect, AspectLock::Original);
        let rotate_active = g.rotate != 0;
        let flip_active = g.flip_h || g.flip_v;
        let persp_active = g.perspective.map(|p| !p.is_identity()).unwrap_or(false);
        if !crop_active
            && !angle_active
            && !aspect_active
            && !rotate_active
            && !flip_active
            && !persp_active
        {
            return None;
        }
        let mut obj = serde_json::Map::new();
        if rotate_active {
            obj.insert("rotate".into(), serde_json::json!(g.rotate));
        }
        if g.flip_h {
            obj.insert("flip_h".into(), serde_json::json!(true));
        }
        if g.flip_v {
            obj.insert("flip_v".into(), serde_json::json!(true));
        }
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
        if persp_active {
            obj.insert(
                "perspective".into(),
                serde_json::to_value(g.perspective?).ok()?,
            );
        }
        Some(serde_json::Value::Object(obj))
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
        if let Some(p) = value.get("perspective") {
            if let Ok(v) = serde_json::from_value::<PerspectiveEdits>(p.clone()) {
                let clamped = v.clamped();
                edits.geometry.perspective = (!clamped.is_identity()).then_some(clamped);
            }
        }
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let g = &edits.geometry;
        let mut steps = g.rotate / 90;
        while steps > 0 {
            let (rotated, nw, nh) = transform::rotate_90(&image.rgb, image.width, image.height);
            image.rgb = rotated;
            image.width = nw;
            image.height = nh;
            steps -= 1;
        }
        if g.flip_h {
            transform::flip_horizontal(&mut image.rgb, image.width, image.height);
        }
        if g.flip_v {
            transform::flip_vertical(&mut image.rgb, image.width, image.height);
        }

        let crop_active = g.crop.map(|c| !c.is_full()).unwrap_or(false);
        let angle_active = g.rotate_angle.abs() > 1e-4;
        let persp_inv = g.perspective_inverse();
        let persp_active = persp_inv != crate::perspective::IDENTITY;
        if !crop_active && !angle_active && !persp_active {
            return Ok(());
        }

        let sw = image.width as f32;
        let sh = image.height as f32;
        let angle = g.rotate_angle;
        let crop = g.crop.unwrap_or(CropRect::full());
        let bbox = geom::rotated_bbox(sw, sh, angle);
        let out_w = (crop.w * bbox.w).round().max(1.0) as usize;
        let out_h = (crop.h * bbox.h).round().max(1.0) as usize;
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
                    let o = geom::display_uv_to_oriented_uv(
                        crop,
                        bbox,
                        sw,
                        sh,
                        angle,
                        &persp_inv,
                        [u, v],
                    );
                    let fx = o[0] * sw - 0.5;
                    let fy = o[1] * sh - 0.5;
                    let d = ox * 3;
                    if fx < 0.0 || fy < 0.0 || fx > (src_w - 1) as f32 || fy > (src_h - 1) as f32 {
                        row[d] = 0.0;
                        row[d + 1] = 0.0;
                        row[d + 2] = 0.0;
                        continue;
                    }
                    let x0 = fx.floor();
                    let y0 = fy.floor();
                    let tx = fx - x0;
                    let ty = fy - y0;
                    let wx = catmull_rom_weights(tx);
                    let wy = catmull_rom_weights(ty);
                    let bx = x0 as isize;
                    let by = y0 as isize;
                    let maxx = src_w as isize - 1;
                    let maxy = src_h as isize - 1;
                    let mut acc = [0.0f32; 3];
                    for (j, wj) in wy.iter().enumerate() {
                        let sy = (by + j as isize - 1).clamp(0, maxy) as usize;
                        for (i, wi) in wx.iter().enumerate() {
                            let sx = (bx + i as isize - 1).clamp(0, maxx) as usize;
                            let w = wi * wj;
                            let s = (sy * src_w + sx) * 3;
                            acc[0] += src[s] * w;
                            acc[1] += src[s + 1] * w;
                            acc[2] += src[s + 2] * w;
                        }
                    }
                    row[d] = acc[0].max(0.0);
                    row[d + 1] = acc[1].max(0.0);
                    row[d + 2] = acc[2].max(0.0);
                }
            });
        image.rgb = out;
        image.width = out_w;
        image.height = out_h;
        Ok(())
    }
    fn gpu(&self) -> Option<GpuOp> {
        Some(GpuOp {
            field_name: "_transform_noop",
            functions: "",
            apply: "",
            vec4_count: 0,
        })
    }
}
