use super::LinearImage;
use super::{GpuOpKind, Op, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::scratch::Scratch;
use crate::edits::Edits;
use crate::frame::RawFrame;
use rayon::prelude::*;

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;

const MIN_SIGMA: f32 = 0.35;
const MAX_SIGMA: f32 = 2.0;
pub const ITERATIONS: usize = 8;
const EPS: f32 = 1e-5;
const CLIP_KNEE: f32 = 0.90;
const CLIP_LIMIT: f32 = 0.98;
const SHADOW_FLOOR: f32 = 0.002;
const SHADOW_KNEE: f32 = 0.02;
const CONTRAST_FLOOR: f32 = 0.010;
const CONTRAST_KNEE: f32 = 0.045;

pub struct CaptureSharpenOp;

impl Op for CaptureSharpenOp {
    fn id(&self) -> &'static str {
        "capture_sharpen"
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
        if edits.detail.capture_sharpen {
            return None;
        }
        Some(serde_json::json!({ "enabled": false }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        if let Some(v) = value.get("enabled").and_then(|v| v.as_bool()) {
            edits.detail.capture_sharpen = v;
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
        let Some(sigma) = context_sigma(ctx, edits) else {
            return Ok(());
        };
        apply_capture_sharpen(image, sigma);
        Ok(())
    }
}

pub fn frame_sigma(frame: &RawFrame, edits: &Edits) -> Option<f32> {
    if !edits.detail.capture_sharpen || !frame.is_raw {
        return None;
    }
    frame
        .capture_sigma
        .filter(|s| *s >= MIN_SIGMA)
        .map(|s| s.min(MAX_SIGMA))
}

pub fn context_sigma(ctx: &OpContext, edits: &Edits) -> Option<f32> {
    if !edits.detail.capture_sharpen || !ctx.render.is_raw {
        return None;
    }
    ctx.render
        .capture_sigma
        .filter(|s| *s >= MIN_SIGMA)
        .map(|s| s.min(MAX_SIGMA))
}

pub fn gaussian_kernel(sigma: f32) -> Vec<f32> {
    let radius = (3.0 * sigma).ceil().max(1.0) as usize;
    let mut kernel: Vec<f32> = (0..=2 * radius)
        .map(|i| {
            let d = i as f32 - radius as f32;
            (-(d * d) / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let sum: f32 = kernel.iter().sum();
    for k in kernel.iter_mut() {
        *k /= sum;
    }
    kernel
}

fn convolve(src: &[f32], dst: &mut [f32], tmp: &mut [f32], w: usize, h: usize, kernel: &[f32]) {
    let radius = kernel.len() / 2;
    let left_end = radius.min(w);
    let right_start = w.saturating_sub(radius).max(left_end);
    tmp.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let src_row = &src[y * w..y * w + w];
        for x in (0..left_end).chain(right_start..w) {
            let mut acc = 0.0f32;
            for (i, k) in kernel.iter().enumerate() {
                let sx = (x + i).saturating_sub(radius).min(w - 1);
                acc += k * src_row[sx];
            }
            row[x] = acc;
        }
        if right_start <= left_end {
            return;
        }
        let inner = &mut row[left_end..right_start];
        inner.fill(0.0);
        for (i, k) in kernel.iter().enumerate() {
            let taps = &src_row[i..i + inner.len()];
            for (out, v) in inner.iter_mut().zip(taps) {
                *out += k * v;
            }
        }
    });
    dst.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        row.fill(0.0);
        for (i, k) in kernel.iter().enumerate() {
            let sy = (y + i).saturating_sub(radius).min(h - 1);
            let src_row = &tmp[sy * w..sy * w + w];
            for (out, v) in row.iter_mut().zip(src_row) {
                *out += k * v;
            }
        }
    });
}

fn separable_extreme(
    src: &[f32],
    dst: &mut [f32],
    tmp: &mut [f32],
    w: usize,
    h: usize,
    radius: usize,
    pick: fn(f32, f32) -> f32,
) {
    tmp.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let src_row = &src[y * w..y * w + w];
        row.copy_from_slice(src_row);
        for d in 1..=radius {
            for (x, out) in row.iter_mut().enumerate() {
                let lo = x.saturating_sub(d);
                let hi = (x + d).min(w - 1);
                *out = pick(pick(*out, src_row[lo]), src_row[hi]);
            }
        }
    });
    dst.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let y0 = y.saturating_sub(radius);
        let y1 = (y + radius).min(h - 1);
        row.copy_from_slice(&tmp[y0 * w..y0 * w + w]);
        for sy in y0 + 1..=y1 {
            let src_row = &tmp[sy * w..sy * w + w];
            for (out, v) in row.iter_mut().zip(src_row) {
                *out = pick(*out, *v);
            }
        }
    });
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn build_blend(image: &LinearImage, luma: &[f32], w: usize, h: usize) -> Scratch {
    let mut blend = Scratch::take_uninit(w * h);
    blend.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        let up = y.saturating_sub(1) * w;
        let down = (y + 1).min(h - 1) * w;
        let mid = y * w;
        for x in 0..w {
            let left = x.saturating_sub(1);
            let right = (x + 1).min(w - 1);
            let gx = luma[mid + right] - luma[mid + left];
            let gy = luma[down + x] - luma[up + x];
            let grad = (gx * gx + gy * gy).sqrt();
            let center = luma[mid + x];
            let peak = image.rgb[(mid + x) * 3]
                .max(image.rgb[(mid + x) * 3 + 1])
                .max(image.rgb[(mid + x) * 3 + 2]);
            row[x] = smoothstep(CONTRAST_FLOOR, CONTRAST_KNEE, grad)
                * smoothstep(SHADOW_FLOOR, SHADOW_KNEE, center)
                * (1.0 - smoothstep(CLIP_KNEE, CLIP_LIMIT, peak));
        }
    });
    blend
}

pub fn apply_capture_sharpen(image: &mut LinearImage, sigma: f32) {
    let w = image.width;
    let h = image.height;
    if w < 8 || h < 8 {
        return;
    }
    let n = w * h;
    let mut luma = Scratch::take_uninit(n);
    luma.par_chunks_mut(w)
        .zip(image.rgb.par_chunks(w * 3))
        .for_each(|(lrow, prow)| {
            for x in 0..w {
                lrow[x] = KR * prow[x * 3] + KG * prow[x * 3 + 1] + KB * prow[x * 3 + 2];
            }
        });
    let blend = build_blend(image, &luma, w, h);
    if blend.par_iter().all(|b| *b <= 0.001) {
        return;
    }
    let kernel = gaussian_kernel(sigma);
    let mut est = Scratch::take_uninit(n);
    est.copy_from_slice(&luma);
    let mut conv = Scratch::take_uninit(n);
    let mut corr = Scratch::take_uninit(n);
    let mut tmp = Scratch::take_uninit(n);
    for _ in 0..ITERATIONS {
        convolve(&est, &mut conv, &mut tmp, w, h, &kernel);
        conv.par_iter_mut().zip(luma.par_iter()).for_each(|(c, l)| {
            *c = l / c.max(EPS);
        });
        convolve(&conv, &mut corr, &mut tmp, w, h, &kernel);
        est.par_iter_mut()
            .zip(corr.par_iter())
            .for_each(|(e, c)| *e *= c);
    }
    let radius = kernel.len() / 2;
    separable_extreme(&luma, &mut conv, &mut tmp, w, h, radius, f32::min);
    separable_extreme(&luma, &mut corr, &mut tmp, w, h, radius, f32::max);
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, prow)| {
            for x in 0..w {
                let i = y * w + x;
                let old = luma[i];
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PreviewMode;
    use crate::ops::{OpScratch, RenderContext};

    fn ctx(is_raw: bool, capture_sigma: Option<f32>) -> OpContext {
        OpContext {
            render: RenderContext {
                wb_coeffs: [1.0; 4],
                cam_to_srgb: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                is_raw,
                capture_sigma,
                preview_mode: PreviewMode::None,
                roi: None,
                dcp: None,
            },
            scratch: OpScratch::default(),
        }
    }

    fn detail_scene(w: usize, h: usize) -> Vec<f32> {
        let mut rgb = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let bar = if (x / 5 + y / 7) % 2 == 0 {
                    0.55f32
                } else {
                    0.18
                };
                let spot = if (x % 23 == 11) && (y % 19 == 9) {
                    0.25
                } else {
                    0.0
                };
                let v = (bar + spot).clamp(0.02, 0.9);
                let i = (y * w + x) * 3;
                rgb[i] = v;
                rgb[i + 1] = v * 0.96;
                rgb[i + 2] = v * 0.92;
            }
        }
        rgb
    }

    fn blur_rgb(rgb: &[f32], w: usize, h: usize, sigma: f32) -> Vec<f32> {
        let kernel = gaussian_kernel(sigma);
        let mut out = rgb.to_vec();
        for c in 0..3 {
            let plane: Vec<f32> = (0..w * h).map(|i| rgb[i * 3 + c]).collect();
            let mut dst = vec![0.0f32; w * h];
            let mut tmp = vec![0.0f32; w * h];
            convolve(&plane, &mut dst, &mut tmp, w, h, &kernel);
            for i in 0..w * h {
                out[i * 3 + c] = dst[i];
            }
        }
        out
    }

    fn rmse(a: &[f32], b: &[f32]) -> f32 {
        let sum: f32 = a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum();
        (sum / a.len() as f32).sqrt()
    }

    #[test]
    fn recovers_detail_lost_to_sensor_blur() {
        let w = 96;
        let h = 96;
        let sharp = detail_scene(w, h);
        for sigma in [0.5f32, 0.8, 1.2] {
            let blurred = blur_rgb(&sharp, w, h, sigma);
            let mut image = LinearImage::new(blurred.clone(), w, h);
            apply_capture_sharpen(&mut image, sigma);
            let before = rmse(&blurred, &sharp);
            let after = rmse(&image.rgb, &sharp);
            eprintln!("sigma {sigma}: rmse {before} -> {after}");
            if after >= before {
                panic!("sigma {sigma}: sharpening did not reduce error ({before} -> {after})");
            }
        }
    }

    #[test]
    fn leaves_flat_noise_alone() {
        let w = 64;
        let h = 64;
        let mut rgb = vec![0.0f32; w * h * 3];
        let mut seed: u32 = 0x9e37_79b9;
        for i in 0..w * h {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let v = 0.35 + (seed as f32 / u32::MAX as f32 - 0.5) * 0.01;
            rgb[i * 3] = v;
            rgb[i * 3 + 1] = v;
            rgb[i * 3 + 2] = v;
        }
        let mut image = LinearImage::new(rgb.clone(), w, h);
        apply_capture_sharpen(&mut image, 0.8);
        let delta = rmse(&image.rgb, &rgb);
        eprintln!("flat noise delta {delta}");
        if delta > 1e-4 {
            panic!("flat noise was amplified by {delta}");
        }
    }

    #[test]
    fn does_not_overshoot_clipped_highlights() {
        let w = 64;
        let h = 64;
        let mut rgb = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let v = if x < w / 2 { 0.05 } else { 1.0 };
                let i = (y * w + x) * 3;
                rgb[i] = v;
                rgb[i + 1] = v;
                rgb[i + 2] = v;
            }
        }
        let blurred = blur_rgb(&rgb, w, h, 1.0);
        let peak_before = blurred.iter().copied().fold(0.0f32, f32::max);
        let mut image = LinearImage::new(blurred, w, h);
        apply_capture_sharpen(&mut image, 1.0);
        let peak_after = image.rgb.iter().copied().fold(0.0f32, f32::max);
        eprintln!("clip peak {peak_before} -> {peak_after}");
        if peak_after > peak_before + 1e-4 {
            panic!("clipped edge overshot from {peak_before} to {peak_after}");
        }
    }

    #[test]
    fn skipped_without_raw_or_sigma() {
        let edits = Edits::default();
        assert!(edits.detail.capture_sharpen);
        assert!(context_sigma(&ctx(true, Some(0.7)), &edits).is_some());
        assert!(context_sigma(&ctx(false, Some(0.7)), &edits).is_none());
        assert!(context_sigma(&ctx(true, None), &edits).is_none());
        assert!(context_sigma(&ctx(true, Some(0.2)), &edits).is_none());
        let mut off = Edits::default();
        off.detail.capture_sharpen = false;
        assert!(context_sigma(&ctx(true, Some(0.7)), &off).is_none());
    }

    #[test]
    fn manifest_round_trip_only_records_the_off_state() {
        let op = CaptureSharpenOp;
        let mut edits = Edits::default();
        assert!(op.to_doc(&edits).is_none());
        edits.detail.capture_sharpen = false;
        let doc = op.to_doc(&edits).expect("doc");
        let mut restored = Edits::default();
        op.from_doc(&doc, &mut restored);
        assert!(!restored.detail.capture_sharpen);
    }
}
