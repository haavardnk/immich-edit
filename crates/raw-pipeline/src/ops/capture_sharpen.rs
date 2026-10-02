mod psf;
#[cfg(test)]
mod tests;

use std::hash::{Hash, Hasher};

use super::LinearImage;
use super::{GpuRoute, Op, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::scratch::Scratch;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::math::{luma, smoothstep};
use psf::{Psf, convolve_with, local_extreme};
use rayon::prelude::*;

pub use psf::{gaussian_kernel, half_diagonal};

const MIN_SIGMA: f32 = 0.35;
const MAX_SIGMA: f32 = 2.0;
const BOOST_PX_PER_UNIT: f32 = 0.01;
pub const SIGMA_LEVELS: usize = 32;
pub const ITERATIONS: usize = 8;
const EPS: f32 = 1e-5;
const CLIP_KNEE: f32 = 0.90;
const CLIP_LIMIT: f32 = 0.98;
const SHADOW_FLOOR: f32 = 0.002;
const SHADOW_KNEE: f32 = 0.02;
const CONTRAST_FLOOR: f32 = 0.010;
const CONTRAST_KNEE: f32 = 0.045;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CaptureSigma {
    pub centre: f32,
    pub corner: f32,
}

impl CaptureSigma {
    pub const fn uniform(sigma: f32) -> Self {
        Self {
            centre: sigma,
            corner: sigma,
        }
    }

    pub fn on_grid(measured: f32, edits: &Edits, block: f32) -> Self {
        let boost = edits.detail.capture_corner_boost as f32 * BOOST_PX_PER_UNIT;
        Self {
            centre: measured / block,
            corner: (measured + boost) / block,
        }
    }

    fn active(self) -> Option<Self> {
        if self.corner < MIN_SIGMA {
            return None;
        }
        Some(Self {
            centre: self.centre.min(MAX_SIGMA),
            corner: self.corner.min(MAX_SIGMA),
        })
    }

    pub fn kernels(&self) -> Vec<Vec<f32>> {
        if self.corner <= self.centre {
            return vec![gaussian_kernel(self.centre)];
        }
        let step = (self.corner - self.centre) / (SIGMA_LEVELS - 1) as f32;
        (0..SIGMA_LEVELS)
            .map(|level| gaussian_kernel(self.centre + step * level as f32))
            .collect()
    }

    pub fn hash_key(&self, h: &mut impl Hasher) {
        self.centre.to_bits().hash(h);
        self.corner.to_bits().hash(h);
    }
}

pub struct CaptureSharpenOp;

impl Op for CaptureSharpenOp {
    fn id(&self) -> &'static str {
        "capture_sharpen"
    }
    fn gpu_route(&self) -> GpuRoute {
        GpuRoute::Detail
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -38
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.capture_sharpen
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let d = &edits.detail;
        if d.capture_sharpen && d.capture_corner_boost == 0.0 {
            return None;
        }
        Some(serde_json::json!({
            "enabled": d.capture_sharpen,
            "corner_boost": d.capture_corner_boost,
        }))
    }
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("enabled").and_then(|v| v.as_bool()) {
            edits.detail.capture_sharpen = v;
        }
        if let Some(v) = value.get("corner_boost").and_then(|v| v.as_f64()) {
            edits.detail.capture_corner_boost = v;
        }
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let Some(sigma) = context_sigma(ctx, edits) else {
            return Ok(());
        };
        apply_capture_sharpen(image, sigma);
        Ok(())
    }
}

pub fn frame_sigma(frame: &RawFrame, edits: &Edits, block: usize) -> Option<CaptureSigma> {
    if !edits.detail.capture_sharpen || !frame.meta.is_raw {
        return None;
    }
    CaptureSigma::on_grid(frame.meta.capture_sigma?, edits, block as f32).active()
}

pub fn context_sigma(ctx: &OpContext, edits: &Edits) -> Option<CaptureSigma> {
    if !edits.detail.capture_sharpen || !ctx.render.is_raw {
        return None;
    }
    ctx.render.capture_sigma?.active()
}

fn build_blend(image: &LinearImage, luma: &[f32], light: &[f32], w: usize, h: usize) -> Scratch {
    let mut blend = Scratch::zeroed(w * h);
    blend.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let up = y.saturating_sub(1) * w;
        let down = (y + 1).min(h - 1) * w;
        let mid = y * w;
        for x in 0..w {
            let left = x.saturating_sub(1);
            let right = (x + 1).min(w - 1);
            let gx = light[mid + right] - light[mid + left];
            let gy = light[down + x] - light[up + x];
            let grad = (gx * gx + gy * gy).sqrt();
            let peak = image.rgb[(mid + x) * 3]
                .max(image.rgb[(mid + x) * 3 + 1])
                .max(image.rgb[(mid + x) * 3 + 2]);
            row[x] = smoothstep(CONTRAST_FLOOR, CONTRAST_KNEE, grad)
                * smoothstep(SHADOW_FLOOR, SHADOW_KNEE, luma[mid + x])
                * (1.0 - smoothstep(CLIP_KNEE, CLIP_LIMIT, peak));
        }
    });
    blend
}

pub fn apply_capture_sharpen(image: &mut LinearImage, sigma: CaptureSigma) {
    let w = image.width;
    let h = image.height;
    if w < 8 || h < 8 {
        return;
    }
    let n = w * h;
    let mut lum = Scratch::zeroed(n);
    lum.par_chunks_mut(w)
        .zip(image.rgb.par_chunks(w * 3))
        .for_each(|(lrow, prow)| {
            for x in 0..w {
                lrow[x] = luma(prow[x * 3], prow[x * 3 + 1], prow[x * 3 + 2]).max(0.0);
            }
        });
    let mut conv = Scratch::zeroed(n);
    conv.par_iter_mut()
        .zip(lum.par_iter())
        .for_each(|(c, l)| *c = l.cbrt());
    let blend = build_blend(image, &lum, &conv, w, h);
    if blend.par_iter().all(|b| *b <= 0.001) {
        return;
    }
    let psf = Psf::new(sigma, w, h);
    let mut est = Scratch::zeroed(n);
    est.copy_from_slice(&lum);
    let mut corr = Scratch::zeroed(n);
    for _ in 0..ITERATIONS {
        convolve_with(&est, &mut conv, w, h, &psf, |y, acc, row| {
            for ((c, a), l) in row.iter_mut().zip(acc).zip(&lum[y * w..y * w + w]) {
                *c = l / a.max(EPS);
            }
        });
        convolve_with(&conv, &mut est, w, h, &psf, |_, acc, row| {
            for (e, a) in row.iter_mut().zip(acc) {
                *e *= a;
            }
        });
    }
    local_extreme(&lum, &mut conv, w, h, &psf, f32::min);
    local_extreme(&lum, &mut corr, w, h, &psf, f32::max);
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, prow)| {
            for x in 0..w {
                let i = y * w + x;
                let old = lum[i];
                if old <= EPS {
                    continue;
                }
                let new = (old + (est[i] - old) * blend[i]).clamp(conv[i], corr[i]);
                let scale = new / old;
                prow[x * 3] *= scale;
                prow[x * 3 + 1] *= scale;
                prow[x * 3 + 2] *= scale;
            }
        });
}
