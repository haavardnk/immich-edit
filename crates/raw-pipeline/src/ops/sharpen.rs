use super::LinearImage;
use super::blur::{gaussian_blur_rgb, gaussian_kernel};
use super::{GpuOpKind, Op, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::scratch::Scratch;
use crate::edits::{DetailEdits, Edits};
use rayon::prelude::*;

pub struct SharpenOp;

impl Op for SharpenOp {
    fn id(&self) -> &'static str {
        "sharpen"
    }
    fn stage(&self) -> Stage {
        Stage::Output
    }
    fn order(&self) -> i32 {
        0
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.sharpen_active() || edits.masked_sharpen_active()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let d = &edits.detail;
        if !d.sharpen_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": d.sharpen_amount,
            "radius": d.sharpen_radius,
            "detail": d.sharpen_detail,
            "masking": d.sharpen_masking,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let d: &mut DetailEdits = &mut edits.detail;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            d.sharpen_amount = v;
        }
        if let Some(v) = value.get("radius").and_then(|v| v.as_f64()) {
            d.sharpen_radius = v;
        }
        if let Some(v) = value.get("detail").and_then(|v| v.as_f64()) {
            d.sharpen_detail = v;
        }
        if let Some(v) = value.get("masking").and_then(|v| v.as_f64()) {
            d.sharpen_masking = v;
        }
    }
    fn gpu_kind(&self) -> GpuOpKind {
        GpuOpKind::Detail
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_sharpen(
            image,
            &edits.detail,
            &ctx.render.preview_mode,
            ctx.scratch.sharpen_delta.as_ref(),
        );
        Ok(())
    }
}

fn apply_sharpen(
    image: &mut LinearImage,
    d: &DetailEdits,
    preview: &crate::frame::PreviewMode,
    delta: Option<&super::SharpenDeltaMap>,
) {
    let amount = (d.sharpen_amount / 25.0) as f32;
    let sigma = d.sharpen_radius as f32;
    let detail_weight = 0.5 + 0.5 * (d.sharpen_detail / 100.0) as f32;
    let masking = (d.sharpen_masking / 100.0) as f32;
    let w = image.width;
    let h = image.height;
    if w < 3 || h < 3 {
        return;
    }
    let kernel = gaussian_kernel(sigma);
    let blur = gaussian_blur_rgb(&image.rgb, w, h, &kernel);
    let mask = if masking > 0.0 || matches!(preview, crate::frame::PreviewMode::SharpenMask) {
        Some(edge_mask(&blur, w, h, masking))
    } else {
        None
    };
    if !matches!(preview, crate::frame::PreviewMode::None) {
        write_preview(image, &blur, mask.as_deref(), detail_weight, preview);
        return;
    }
    let strength = amount * detail_weight;
    let base_amount = d.sharpen_amount as f32;
    image
        .rgb
        .par_chunks_mut(w * 3)
        .zip(blur.par_chunks(w * 3))
        .enumerate()
        .for_each(|(y, (row, brow))| {
            for x in 0..w {
                let i = x * 3;
                let m = match &mask {
                    Some(m) => m[y * w + x],
                    None => 1.0,
                };
                let k = match delta {
                    Some(map) => {
                        let amt = (base_amount + map.sample(x, y, w, h)).clamp(-150.0, 150.0);
                        amt / 25.0 * detail_weight * m
                    }
                    None => strength * m,
                };
                for c in 0..3 {
                    let v = row[i + c];
                    let high = v - brow[i + c];
                    row[i + c] = v + k * high;
                }
            }
        });
}

fn edge_mask(blur: &[f32], w: usize, h: usize, masking: f32) -> Scratch {
    let mut luma = Scratch::take_uninit(w * h);
    luma.par_chunks_mut(w)
        .zip(blur.par_chunks(w * 3))
        .for_each(|(lrow, brow)| {
            for (x, slot) in lrow.iter_mut().enumerate() {
                let i = x * 3;
                *slot = 0.2126 * brow[i] + 0.7152 * brow[i + 1] + 0.0722 * brow[i + 2];
            }
        });
    let mut mag = Scratch::take_uninit(w * h);
    mag.par_chunks_mut(w).enumerate().for_each(|(y, mrow)| {
        let ym1 = y.saturating_sub(1);
        let yp1 = (y + 1).min(h - 1);
        for x in 0..w {
            let xm1 = x.saturating_sub(1);
            let xp1 = (x + 1).min(w - 1);
            let gx = -luma[ym1 * w + xm1] - 2.0 * luma[y * w + xm1] - luma[yp1 * w + xm1]
                + luma[ym1 * w + xp1]
                + 2.0 * luma[y * w + xp1]
                + luma[yp1 * w + xp1];
            let gy = -luma[ym1 * w + xm1] - 2.0 * luma[ym1 * w + x] - luma[ym1 * w + xp1]
                + luma[yp1 * w + xm1]
                + 2.0 * luma[yp1 * w + x]
                + luma[yp1 * w + xp1];
            mrow[x] = (gx * gx + gy * gy).sqrt();
        }
    });
    let thresh = masking * 0.15;
    let width = 0.15f32;
    mag.par_iter_mut().for_each(|m| {
        let t = ((*m - thresh) / width).clamp(0.0, 1.0);
        *m = t * t * (3.0 - 2.0 * t);
    });
    mag
}

fn write_preview(
    image: &mut LinearImage,
    blur: &[f32],
    mask: Option<&[f32]>,
    detail_weight: f32,
    preview: &crate::frame::PreviewMode,
) {
    let w = image.width;
    image
        .rgb
        .par_chunks_mut(w * 3)
        .zip(blur.par_chunks(w * 3))
        .enumerate()
        .for_each(|(y, (row, brow))| {
            for x in 0..w {
                let i = x * 3;
                let v = match preview {
                    crate::frame::PreviewMode::SharpenMask => match mask {
                        Some(m) => m[y * w + x],
                        None => 0.0,
                    },
                    crate::frame::PreviewMode::SharpenRadius => {
                        0.2126 * brow[i] + 0.7152 * brow[i + 1] + 0.0722 * brow[i + 2]
                    }
                    crate::frame::PreviewMode::SharpenDetail => {
                        let lr = 0.2126 * row[i] + 0.7152 * row[i + 1] + 0.0722 * row[i + 2];
                        let lb = 0.2126 * brow[i] + 0.7152 * brow[i + 1] + 0.0722 * brow[i + 2];
                        (8.0 * detail_weight * (lr - lb).abs()).clamp(0.0, 1.0)
                    }
                    _ => row[i],
                };
                row[i] = v;
                row[i + 1] = v;
                row[i + 2] = v;
            }
        });
}
