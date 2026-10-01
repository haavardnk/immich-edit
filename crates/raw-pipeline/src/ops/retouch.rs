use super::LinearImage;
use super::sample::sample_rgb_bicubic;
use super::{Op, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, RetouchMode, RetouchStroke};
use rayon::prelude::*;

pub struct RetouchOp;

impl Op for RetouchOp {
    fn id(&self) -> &'static str {
        "retouch"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Pass(super::GpuPass::Retouch)
    }
    fn stage(&self) -> Stage {
        Stage::WhiteBalance
    }
    fn order(&self) -> i32 {
        30
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.retouch.iter().any(|s| s.is_effective())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        if edits.retouch.is_empty() {
            return None;
        }
        Some(serde_json::json!({ "strokes": edits.retouch }))
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let Some(arr) = value.get("strokes").and_then(|v| v.as_array()) else {
            return;
        };
        edits.retouch = arr
            .iter()
            .filter_map(|item| serde_json::from_value(item.clone()).ok())
            .collect();
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        for stroke in &edits.retouch {
            if stroke.is_effective() {
                apply_stroke(image, stroke);
            }
        }
        Ok(())
    }
}

const SAMPLE_MARGIN: f32 = 2.0;
const TENT: [f32; 4] = [0.125, 0.375, 0.375, 0.125];

pub(crate) struct Bbox {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

pub(crate) struct StrokeGeom {
    pub points: Vec<(f32, f32)>,
    pub radius_px: f32,
    pub off_x: f32,
    pub off_y: f32,
    pub bbox: Bbox,
}

pub(crate) fn stroke_geometry(
    stroke: &RetouchStroke,
    w: usize,
    h: usize,
    orient: crate::frame::OrientFlips,
) -> Option<StrokeGeom> {
    if w < 2 || h < 2 || stroke.points.is_empty() {
        return None;
    }
    let map = |p: &crate::edits::Vec2f| {
        let (t, hf, vf) = orient;
        let mut u = p.x;
        let mut v = p.y;
        if t {
            std::mem::swap(&mut u, &mut v);
        }
        if hf {
            u = 1.0 - u;
        }
        if vf {
            v = 1.0 - v;
        }
        (u * w as f32, v * h as f32)
    };
    let points: Vec<(f32, f32)> = stroke.points.iter().map(map).collect();
    let scale = w.min(h) as f32;
    let radius_px = stroke.radius * scale;
    if radius_px < 0.5 {
        return None;
    }
    let bbox = stroke_bbox(&points, radius_px, w, h)?;
    let cx = points.iter().map(|p| p.0).sum::<f32>() / points.len() as f32;
    let cy = points.iter().map(|p| p.1).sum::<f32>() / points.len() as f32;
    let source = map(&stroke.source);
    let off_x = clamp_offset(source.0 - cx, bbox.x0, bbox.x1, w);
    let off_y = clamp_offset(source.1 - cy, bbox.y0, bbox.y1, h);
    if off_x.abs() < 0.5 && off_y.abs() < 0.5 {
        return None;
    }
    Some(StrokeGeom {
        points,
        radius_px,
        off_x,
        off_y,
        bbox,
    })
}

fn clamp_offset(off: f32, lo: usize, hi: usize, extent: usize) -> f32 {
    let min_off = SAMPLE_MARGIN - lo as f32;
    let max_off = (extent as f32 - SAMPLE_MARGIN) - hi as f32;
    if min_off > max_off {
        return (min_off + max_off) * 0.5;
    }
    off.clamp(min_off, max_off)
}

fn stroke_bbox(points: &[(f32, f32)], radius_px: f32, w: usize, h: usize) -> Option<Bbox> {
    let pad = radius_px + 2.0;
    let min_x = points.iter().map(|p| p.0).fold(f32::MAX, f32::min) - pad;
    let max_x = points.iter().map(|p| p.0).fold(f32::MIN, f32::max) + pad;
    let min_y = points.iter().map(|p| p.1).fold(f32::MAX, f32::min) - pad;
    let max_y = points.iter().map(|p| p.1).fold(f32::MIN, f32::max) + pad;
    let x0 = min_x.floor().max(0.0) as usize;
    let y0 = min_y.floor().max(0.0) as usize;
    let x1 = (max_x.ceil() as isize).clamp(0, w as isize) as usize;
    let y1 = (max_y.ceil() as isize).clamp(0, h as isize) as usize;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Bbox { x0, y0, x1, y1 })
}

#[inline]
fn point_polyline_distance(px: f32, py: f32, points: &[(f32, f32)]) -> f32 {
    if points.len() == 1 {
        let dx = px - points[0].0;
        let dy = py - points[0].1;
        return (dx * dx + dy * dy).sqrt();
    }
    let mut best = f32::MAX;
    for pair in points.windows(2) {
        let (ax, ay) = pair[0];
        let (bx, by) = pair[1];
        let dx = bx - ax;
        let dy = by - ay;
        let len2 = dx * dx + dy * dy;
        let t = if len2 <= 1e-12 {
            0.0
        } else {
            (((px - ax) * dx + (py - ay) * dy) / len2).clamp(0.0, 1.0)
        };
        let cx = ax + t * dx;
        let cy = ay + t * dy;
        let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
        if d < best {
            best = d;
        }
    }
    best
}

#[inline]
pub(crate) fn coverage(dist: f32, radius_px: f32, hardness: f32) -> f32 {
    if dist >= radius_px {
        return 0.0;
    }
    let inner = radius_px * hardness.clamp(0.0, 1.0);
    if dist <= inner {
        return 1.0;
    }
    let falloff = radius_px - inner;
    if falloff <= 1e-6 {
        return 1.0;
    }
    let t = (radius_px - dist) / falloff;
    t * t * (3.0 - 2.0 * t)
}

fn apply_stroke(image: &mut LinearImage, stroke: &RetouchStroke) {
    let w = image.width;
    let h = image.height;
    let Some(geom) = stroke_geometry(stroke, w, h, (false, false, false)) else {
        return;
    };
    let points = geom.points;
    let radius_px = geom.radius_px;
    let off_x = geom.off_x;
    let off_y = geom.off_y;
    let bb = geom.bbox;

    let pw = bb.x1 - bb.x0;
    let ph = bb.y1 - bb.y0;
    let mut src_patch = vec![0.0f32; pw * ph * 3];
    src_patch
        .par_chunks_mut(pw * 3)
        .enumerate()
        .for_each(|(row, dst)| {
            let sy = (bb.y0 + row) as f32 + off_y;
            for x in 0..pw {
                let sx = (bb.x0 + x) as f32 + off_x;
                let rgb = sample_rgb_bicubic(&image.rgb, w, h, sx, sy);
                let i = x * 3;
                dst[i] = rgb[0];
                dst[i + 1] = rgb[1];
                dst[i + 2] = rgb[2];
            }
        });

    let heal = matches!(stroke.mode, RetouchMode::Heal);
    let residual = heal.then(|| {
        let mut known = vec![[0.0f32; 4]; pw * ph];
        known.par_chunks_mut(pw).enumerate().for_each(|(row, out)| {
            let y = bb.y0 + row;
            for (x, px) in out.iter_mut().enumerate() {
                let gx = bb.x0 + x;
                let d = point_polyline_distance(gx as f32 + 0.5, y as f32 + 0.5, &points);
                if d < radius_px {
                    continue;
                }
                let i = (y * w + gx) * 3;
                let p = (row * pw + x) * 3;
                *px = [
                    image.rgb[i] - src_patch[p],
                    image.rgb[i + 1] - src_patch[p + 1],
                    image.rgb[i + 2] - src_patch[p + 2],
                    1.0,
                ];
            }
        });
        membrane_fill(Level {
            w: pw,
            h: ph,
            px: known,
        })
    });

    let opacity = stroke.opacity;
    let hardness = stroke.hardness;
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .skip(bb.y0)
        .take(ph)
        .for_each(|(y, row)| {
            let prow = y - bb.y0;
            for x in bb.x0..bb.x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let d = point_polyline_distance(px, py, &points);
                let cov = coverage(d, radius_px, hardness) * opacity;
                if cov <= 0.0 {
                    continue;
                }
                let pi = (prow * pw + (x - bb.x0)) * 3;
                let i = x * 3;
                for c in 0..3 {
                    let source = match &residual {
                        Some(r) => (src_patch[pi + c] + r[pi / 3][c]).max(0.0),
                        None => src_patch[pi + c],
                    };
                    row[i + c] = row[i + c] * (1.0 - cov) + source * cov;
                }
            }
        });
}

struct Level {
    w: usize,
    h: usize,
    px: Vec<[f32; 4]>,
}

fn membrane_fill(base: Level) -> Vec<[f32; 4]> {
    let mut pyramid = vec![base];
    while let Some(fine) = pyramid.last().filter(|l| l.w > 1 || l.h > 1) {
        let next = push_level(fine);
        pyramid.push(next);
    }
    let mut fill: Option<Level> = None;
    for level in pyramid.iter().rev() {
        let coarse = fill.as_ref().unwrap_or(level);
        fill = Some(pull_level(level, coarse));
    }
    fill.map(|l| l.px).unwrap_or_default()
}

fn push_level(fine: &Level) -> Level {
    let w = (fine.w / 2).max(1);
    let h = (fine.h / 2).max(1);
    let px = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let cx = (i % w) as isize;
            let cy = (i / w) as isize;
            let mut acc = [0.0f32; 4];
            for (j, wy) in TENT.iter().enumerate() {
                let fy = (2 * cy - 1 + j as isize).clamp(0, fine.h as isize - 1) as usize;
                for (k, wx) in TENT.iter().enumerate() {
                    let fx = (2 * cx - 1 + k as isize).clamp(0, fine.w as isize - 1) as usize;
                    let p = fine.px[fy * fine.w + fx];
                    for (a, v) in acc.iter_mut().zip(p) {
                        *a += wx * wy * v;
                    }
                }
            }
            acc
        })
        .collect();
    Level { w, h, px }
}

fn pull_level(fine: &Level, coarse: &Level) -> Level {
    let at = |x: isize, y: isize| -> [f32; 3] {
        let x = x.clamp(0, coarse.w as isize - 1) as usize;
        let y = y.clamp(0, coarse.h as isize - 1) as usize;
        let p = coarse.px[y * coarse.w + x];
        let a = p[3].max(1e-8);
        [p[0] / a, p[1] / a, p[2] / a]
    };
    let px = (0..fine.w * fine.h)
        .into_par_iter()
        .map(|i| {
            let u = ((i % fine.w) as f32 + 0.5) * 0.5 - 0.5;
            let v = ((i / fine.w) as f32 + 0.5) * 0.5 - 0.5;
            let tx = u - u.floor();
            let ty = v - v.floor();
            let x0 = u.floor() as isize;
            let y0 = v.floor() as isize;
            let c00 = at(x0, y0);
            let c10 = at(x0 + 1, y0);
            let c01 = at(x0, y0 + 1);
            let c11 = at(x0 + 1, y0 + 1);
            let f = fine.px[i];
            let mut out = [0.0f32, 0.0, 0.0, 1.0];
            for c in 0..3 {
                let top = c00[c] + (c10[c] - c00[c]) * tx;
                let bottom = c01[c] + (c11[c] - c01[c]) * tx;
                let up = top + (bottom - top) * ty;
                out[c] = f[c] + (1.0 - f[3]) * up;
            }
            out
        })
        .collect();
    Level {
        w: fine.w,
        h: fine.h,
        px,
    }
}
