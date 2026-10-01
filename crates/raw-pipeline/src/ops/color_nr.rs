use super::LinearImage;
use super::denoise;
use super::{GpuRoute, Op, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{DetailEdits, Edits};

pub struct ColorNrOp;

impl Op for ColorNrOp {
    fn id(&self) -> &'static str {
        "color_nr"
    }
    fn gpu_route(&self) -> GpuRoute {
        GpuRoute::Detail
    }
    fn stage(&self) -> Stage {
        Stage::Tone
    }
    fn order(&self) -> i32 {
        -50
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.detail.color_nr_active()
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
    fn apply_doc(&self, value: &serde_json::Value, edits: &mut Edits) {
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
        denoise::chroma::denoise(
            image,
            denoise::chroma::level_params(
                d.color_nr_amount as f32,
                d.color_nr_detail as f32,
                d.color_nr_smoothness as f32,
            ),
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edits::Edits;
    use crate::frame::PreviewMode;
    use crate::math::luma;
    use crate::ops::denoise::{PB_DEN, PR_DEN};
    use crate::ops::{OpContext, OpScratch, RenderContext};
    use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};

    fn ctx() -> OpContext {
        OpContext {
            render: RenderContext {
                wb_coeffs: [1.0; 4],
                cam_to_srgb: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                is_raw: false,
                capture_sigma: None,
                preview_mode: PreviewMode::None,
                roi: None,
                dcp: None,
            },
            scratch: OpScratch::default(),
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
                let yv = LUMA_R * r + LUMA_G * 0.5 + LUMA_B * b;
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
            s += luma(px[0], px[1], px[2]);
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
