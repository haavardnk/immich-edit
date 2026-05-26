use super::LinearImage;
use super::{GpuOpKind, OpContext, OpMeta, SpatialOp, Stage};
use crate::PipelineResult;
use crate::cpu::scratch::Scratch;
use crate::edits::{DetailEdits, Edits};
use rayon::prelude::*;

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;

pub struct LumaNrOp;

impl OpMeta for LumaNrOp {
    fn id(&self) -> &'static str {
        "luma_nr"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -50
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.luma_nr_active()
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let d = &edits.detail;
        if !d.luma_nr_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": d.luma_nr_amount,
            "detail": d.luma_nr_detail,
            "contrast": d.luma_nr_contrast,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let d: &mut DetailEdits = &mut edits.detail;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            d.luma_nr_amount = v;
        }
        if let Some(v) = value.get("detail").and_then(|v| v.as_f64()) {
            d.luma_nr_detail = v;
        }
        if let Some(v) = value.get("contrast").and_then(|v| v.as_f64()) {
            d.luma_nr_contrast = v;
        }
    }
}

impl SpatialOp for LumaNrOp {
    fn gpu_kind(&self) -> GpuOpKind {
        GpuOpKind::Detail
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        let d = &edits.detail;
        if !d.luma_nr_active() {
            return Ok(());
        }
        apply_luma_nr(
            image,
            d.luma_nr_amount as f32,
            d.luma_nr_detail as f32,
            d.luma_nr_contrast as f32,
        );
        Ok(())
    }
}

fn apply_luma_nr(image: &mut LinearImage, amount: f32, detail: f32, contrast: f32) {
    let w = image.width;
    let h = image.height;
    if w < 3 || h < 3 {
        return;
    }
    let n = w * h;
    let mut luma = Scratch::take_uninit(n);
    luma.par_chunks_mut(w)
        .zip(image.rgb.par_chunks(w * 3))
        .for_each(|(lrow, prow)| {
            for x in 0..w {
                let r = prow[x * 3];
                let g = prow[x * 3 + 1];
                let b = prow[x * 3 + 2];
                lrow[x] = KR * r + KG * g + KB * b;
            }
        });
    let radius: i32 = if amount >= 66.0 {
        4
    } else if amount >= 33.0 {
        3
    } else {
        2
    };
    let sigma_s = radius as f32;
    let sigma_r = 0.005 + (1.0 - detail / 100.0) * 0.20;
    let inv_2ss = 1.0 / (2.0 * sigma_s * sigma_s);
    let inv_2sr = 1.0 / (2.0 * sigma_r * sigma_r);
    let alpha = (amount / 100.0) * (1.0 - contrast / 100.0);
    let mut denoised = Scratch::take_uninit(n);
    denoised
        .par_chunks_mut(w)
        .enumerate()
        .for_each(|(y, drow)| {
            for x in 0..w {
                let center = luma[y * w + x];
                let mut wsum = 0.0f32;
                let mut acc = 0.0f32;
                let y0 = (y as i32 - radius).max(0) as usize;
                let y1 = (y as i32 + radius).min(h as i32 - 1) as usize;
                let x0 = (x as i32 - radius).max(0) as usize;
                let x1 = (x as i32 + radius).min(w as i32 - 1) as usize;
                for yy in y0..=y1 {
                    for xx in x0..=x1 {
                        let v = luma[yy * w + xx];
                        let dx = xx as f32 - x as f32;
                        let dy = yy as f32 - y as f32;
                        let dr = v - center;
                        let wgt = (-(dx * dx + dy * dy) * inv_2ss - dr * dr * inv_2sr).exp();
                        wsum += wgt;
                        acc += wgt * v;
                    }
                }
                drow[x] = if wsum > 0.0 { acc / wsum } else { center };
            }
        });
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, prow)| {
            for x in 0..w {
                let y_orig = luma[y * w + x];
                let y_den = denoised[y * w + x];
                let y_new = y_orig + (y_den - y_orig) * alpha;
                let scale = if y_orig > 1e-6 { y_new / y_orig } else { 1.0 };
                prow[x * 3] *= scale;
                prow[x * 3 + 1] *= scale;
                prow[x * 3 + 2] *= scale;
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::Edits;
    use crate::frame::PreviewMode;
    use crate::ops::{OpContext, OpScratch, RenderContext};

    fn ctx() -> OpContext {
        OpContext {
            render: RenderContext {
                wb_coeffs: [1.0; 4],
                cam_to_srgb: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                is_raw: false,
                preview_mode: PreviewMode::None,
            },
            scratch: OpScratch { shadows_blur: None },
        }
    }

    fn noisy_image(w: usize, h: usize) -> LinearImage {
        let mut rgb = vec![0.0f32; w * h * 3];
        let mut seed: u32 = 0x1234_5678;
        for i in 0..w * h {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let n = (seed as f32 / u32::MAX as f32 - 0.5) * 0.10;
            let base = if (i / w) < h / 2 { 0.2 } else { 0.8 };
            rgb[i * 3] = (base + n).clamp(0.0, 1.0);
            rgb[i * 3 + 1] = (base + n).clamp(0.0, 1.0);
            rgb[i * 3 + 2] = (base + n).clamp(0.0, 1.0);
        }
        LinearImage::new(rgb, w, h)
    }

    fn high_freq_variance(image: &LinearImage) -> f32 {
        let w = image.width;
        let h = image.height;
        let mut sum = 0.0f32;
        let mut count = 0usize;
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let c = image.rgb[(y * w + x) * 3];
                let n = image.rgb[((y - 1) * w + x) * 3];
                let s = image.rgb[((y + 1) * w + x) * 3];
                let e = image.rgb[(y * w + x + 1) * 3];
                let w_ = image.rgb[(y * w + x - 1) * 3];
                let avg = (n + s + e + w_) * 0.25;
                let d = c - avg;
                sum += d * d;
                count += 1;
            }
        }
        sum / count as f32
    }

    #[test]
    fn amount_zero_identity() {
        let mut img = noisy_image(16, 16);
        let snapshot = img.rgb.clone();
        let mut edits = Edits::default();
        edits.detail.luma_nr_amount = 0.0;
        LumaNrOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        for (a, b) in img.rgb.iter().zip(snapshot.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn reduces_noise_preserves_edge() {
        let mut img = noisy_image(48, 48);
        let before = high_freq_variance(&img);
        let mut edits = Edits::default();
        edits.detail.luma_nr_amount = 80.0;
        edits.detail.luma_nr_detail = 30.0;
        edits.detail.luma_nr_contrast = 0.0;
        LumaNrOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        let after = high_freq_variance(&img);
        assert!(after < before * 0.6, "before={before} after={after}");
        let top = img.rgb[(8 * 48 + 24) * 3];
        let bot = img.rgb[(40 * 48 + 24) * 3];
        assert!(bot - top > 0.4, "edge collapsed top={top} bot={bot}");
    }
}
