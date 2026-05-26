use super::LinearImage;
use super::{EditOperator, GpuOpKind, OpContext, Stage};
use crate::PipelineResult;
use crate::cpu::scratch::Scratch;
use crate::edits::{DetailEdits, Edits};
use rayon::prelude::*;

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;
const PB_DEN: f32 = 1.8556;
const PR_DEN: f32 = 1.5748;

pub struct ColorNrOp;

impl EditOperator for ColorNrOp {
    fn id(&self) -> &'static str {
        "color_nr"
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -40
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.color_nr_active()
    }
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
        if !d.color_nr_active() {
            return Ok(());
        }
        apply_color_nr(
            image,
            d.color_nr_amount as f32,
            d.color_nr_detail as f32,
            d.color_nr_smoothness as f32,
        );
        Ok(())
    }
    fn to_doc(&self, edits: &Edits) -> Option<serde_json::Value> {
        let d = &edits.detail;
        if !d.color_nr_active() {
            return None;
        }
        Some(serde_json::json!({
            "amount": d.color_nr_amount,
            "detail": d.color_nr_detail,
            "smoothness": d.color_nr_smoothness,
        }))
    }
    fn from_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
        let d: &mut DetailEdits = &mut edits.detail;
        if let Some(v) = value.get("amount").and_then(|v| v.as_f64()) {
            d.color_nr_amount = v;
        }
        if let Some(v) = value.get("detail").and_then(|v| v.as_f64()) {
            d.color_nr_detail = v;
        }
        if let Some(v) = value.get("smoothness").and_then(|v| v.as_f64()) {
            d.color_nr_smoothness = v;
        }
    }
}

fn apply_color_nr(image: &mut LinearImage, amount: f32, detail: f32, smoothness: f32) {
    let w = image.width;
    let h = image.height;
    if w < 3 || h < 3 {
        return;
    }
    let n = w * h;
    let mut y_buf = Scratch::take_uninit(n);
    let mut pb_buf = Scratch::take_uninit(n);
    let mut pr_buf = Scratch::take_uninit(n);
    (
        y_buf.par_chunks_mut(w),
        pb_buf.par_chunks_mut(w),
        pr_buf.par_chunks_mut(w),
    )
        .into_par_iter()
        .zip(image.rgb.par_chunks(w * 3))
        .for_each(|((yrow, pbrow, prrow), prow)| {
            for x in 0..w {
                let r = prow[x * 3];
                let g = prow[x * 3 + 1];
                let b = prow[x * 3 + 2];
                let yv = KR * r + KG * g + KB * b;
                yrow[x] = yv;
                pbrow[x] = (b - yv) / PB_DEN;
                prrow[x] = (r - yv) / PR_DEN;
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
    let sigma_r = 0.005 + (1.0 - detail / 100.0) * 0.30;
    let inv_2ss = 1.0 / (2.0 * sigma_s * sigma_s);
    let inv_2sr = 1.0 / (2.0 * sigma_r * sigma_r);
    let alpha = amount / 100.0;
    let mut pb_out = Scratch::take_uninit(n);
    let mut pr_out = Scratch::take_uninit(n);
    (pb_out.par_chunks_mut(w), pr_out.par_chunks_mut(w))
        .into_par_iter()
        .enumerate()
        .for_each(|(y, (pbrow, prrow))| {
            for x in 0..w {
                let cb = pb_buf[y * w + x];
                let cr = pr_buf[y * w + x];
                let mut wsum = 0.0f32;
                let mut acc_b = 0.0f32;
                let mut acc_r = 0.0f32;
                let y0 = (y as i32 - radius).max(0) as usize;
                let y1 = (y as i32 + radius).min(h as i32 - 1) as usize;
                let x0 = (x as i32 - radius).max(0) as usize;
                let x1 = (x as i32 + radius).min(w as i32 - 1) as usize;
                for yy in y0..=y1 {
                    for xx in x0..=x1 {
                        let vb = pb_buf[yy * w + xx];
                        let vr = pr_buf[yy * w + xx];
                        let dx = xx as f32 - x as f32;
                        let dy = yy as f32 - y as f32;
                        let dcb = vb - cb;
                        let dcr = vr - cr;
                        let dr2 = dcb * dcb + dcr * dcr;
                        let wgt = (-(dx * dx + dy * dy) * inv_2ss - dr2 * inv_2sr).exp();
                        wsum += wgt;
                        acc_b += wgt * vb;
                        acc_r += wgt * vr;
                    }
                }
                if wsum > 0.0 {
                    pbrow[x] = acc_b / wsum;
                    prrow[x] = acc_r / wsum;
                } else {
                    pbrow[x] = cb;
                    prrow[x] = cr;
                }
            }
        });
    let s = smoothness / 100.0;
    if s > 0.0 {
        let mut pb_s = Scratch::take_uninit(n);
        let mut pr_s = Scratch::take_uninit(n);
        box_blur_3x3(&pb_out, &mut pb_s, w, h);
        box_blur_3x3(&pr_out, &mut pr_s, w, h);
        pb_out
            .par_iter_mut()
            .zip(pb_s.par_iter())
            .for_each(|(a, b)| *a = *a + (*b - *a) * s);
        pr_out
            .par_iter_mut()
            .zip(pr_s.par_iter())
            .for_each(|(a, b)| *a = *a + (*b - *a) * s);
    }
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, prow)| {
            for x in 0..w {
                let yv = y_buf[y * w + x];
                let pb_orig = pb_buf[y * w + x];
                let pr_orig = pr_buf[y * w + x];
                let pb_new = pb_orig + (pb_out[y * w + x] - pb_orig) * alpha;
                let pr_new = pr_orig + (pr_out[y * w + x] - pr_orig) * alpha;
                let r = yv + PR_DEN * pr_new;
                let b = yv + PB_DEN * pb_new;
                let g = (yv - KR * r - KB * b) / KG;
                prow[x * 3] = r;
                prow[x * 3 + 1] = g;
                prow[x * 3 + 2] = b;
            }
        });
}

fn box_blur_3x3(src: &[f32], dst: &mut [f32], w: usize, h: usize) {
    dst.par_chunks_mut(w).enumerate().for_each(|(y, drow)| {
        let y0 = if y == 0 { 0 } else { y - 1 };
        let y1 = if y + 1 >= h { h - 1 } else { y + 1 };
        for (x, slot) in drow.iter_mut().enumerate() {
            let x0 = if x == 0 { 0 } else { x - 1 };
            let x1 = if x + 1 >= w { w - 1 } else { x + 1 };
            let mut sum = 0.0f32;
            let mut cnt = 0.0f32;
            for yy in y0..=y1 {
                for xx in x0..=x1 {
                    sum += src[yy * w + xx];
                    cnt += 1.0;
                }
            }
            *slot = sum / cnt;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::Edits;
    use crate::frame::PreviewMode;
    use crate::ops::OpContext;

    fn ctx() -> OpContext {
        OpContext {
            wb_coeffs: [1.0; 4],
            cam_to_srgb: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            is_raw: false,
            preview_mode: PreviewMode::None,
            shadows_blur: None,
        }
    }

    fn chroma_noise_image(w: usize, h: usize) -> LinearImage {
        let mut rgb = vec![0.0f32; w * h * 3];
        let mut seed: u32 = 0xCAFE_BABE;
        for i in 0..w * h {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let nr = (seed as f32 / u32::MAX as f32 - 0.5) * 0.20;
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let nb = (seed as f32 / u32::MAX as f32 - 0.5) * 0.20;
            rgb[i * 3] = (0.5 + nr).clamp(0.0, 1.0);
            rgb[i * 3 + 1] = 0.5;
            rgb[i * 3 + 2] = (0.5 + nb).clamp(0.0, 1.0);
        }
        LinearImage::new(rgb, w, h)
    }

    fn chroma_variance(image: &LinearImage) -> f32 {
        let w = image.width;
        let h = image.height;
        let mut sum = 0.0f32;
        let mut count = 0usize;
        for y in 0..h {
            for x in 0..w {
                let r = image.rgb[(y * w + x) * 3];
                let b = image.rgb[(y * w + x) * 3 + 2];
                let yv = KR * r + KG * 0.5 + KB * b;
                let cb = (b - yv) / PB_DEN;
                let cr = (r - yv) / PR_DEN;
                sum += cb * cb + cr * cr;
                count += 1;
            }
        }
        sum / count as f32
    }

    fn luma_mean(image: &LinearImage) -> f32 {
        let mut s = 0.0f32;
        for px in image.rgb.chunks(3) {
            s += KR * px[0] + KG * px[1] + KB * px[2];
        }
        s / (image.rgb.len() / 3) as f32
    }

    #[test]
    fn amount_zero_identity() {
        let mut img = chroma_noise_image(16, 16);
        let snapshot = img.rgb.clone();
        let mut edits = Edits::default();
        edits.detail.color_nr_amount = 0.0;
        ColorNrOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        for (a, b) in img.rgb.iter().zip(snapshot.iter()) {
            assert!((a - b).abs() < 1e-6);
        }
    }

    #[test]
    fn reduces_chroma_preserves_luma() {
        let mut img = chroma_noise_image(48, 48);
        let cv_before = chroma_variance(&img);
        let y_before = luma_mean(&img);
        let mut edits = Edits::default();
        edits.detail.color_nr_amount = 80.0;
        edits.detail.color_nr_detail = 30.0;
        edits.detail.color_nr_smoothness = 0.0;
        ColorNrOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        let cv_after = chroma_variance(&img);
        let y_after = luma_mean(&img);
        assert!(
            cv_after < cv_before * 0.5,
            "before={cv_before} after={cv_after}"
        );
        assert!((y_after - y_before).abs() < 0.01);
    }
}
