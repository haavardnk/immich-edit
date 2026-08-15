use super::LinearImage;
use super::{Op, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, LensEdits};
use rayon::prelude::*;

pub struct LensVignetteOp;

impl Op for LensVignetteOp {
    fn id(&self) -> &'static str {
        "lens_vignette"
    }
    fn gpu_route(&self) -> super::GpuRoute {
        super::GpuRoute::Pass(super::GpuPass::Lens)
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn order(&self) -> i32 {
        1
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens.vignette_active()
    }
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_lens_vignette(image, &edits.lens);
        Ok(())
    }
}

pub fn vignette_coeffs(lens: &LensEdits) -> (f32, f32, f32, f32) {
    if !lens.profile_active() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let amount = (lens.vignette_amount / 100.0) as f32 * VIGNETTE_AMOUNT_DAMPENING;
    (lens.vk1 as f32, lens.vk2 as f32, lens.vk3 as f32, amount)
}

pub const VIGNETTE_GAIN_MIN: f32 = 0.25;
pub const VIGNETTE_GAIN_MAX: f32 = 8.0;
pub const VIGNETTE_POLY_MIN: f32 = 1.0 / VIGNETTE_GAIN_MAX;
pub const VIGNETTE_AMOUNT_DAMPENING: f32 = 0.8;

#[inline]
pub fn vignette_correction(vk1: f32, vk2: f32, vk3: f32, amount: f32, r_norm: f32) -> f32 {
    let r2 = r_norm * r_norm;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let poly = (1.0 + vk1 * r2 + vk2 * r4 + vk3 * r6).max(VIGNETTE_POLY_MIN);
    let full_gain = 1.0 / poly;
    let gain = 1.0 + (full_gain - 1.0) * amount;
    gain.clamp(VIGNETTE_GAIN_MIN, VIGNETTE_GAIN_MAX)
}

pub fn apply_lens_vignette(image: &mut LinearImage, lens: &LensEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let (vk1, vk2, vk3, amount) = vignette_coeffs(lens);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let half_diag = 0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
    let inv_diag = 1.0 / half_diag;

    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let dy = y as f32 + 0.5 - cy;
            for x in 0..w {
                let dx = x as f32 + 0.5 - cx;
                let r = (dx * dx + dy * dy).sqrt() * inv_diag;
                let correction = vignette_correction(vk1, vk2, vk3, amount, r);
                let i = x * 3;
                row[i] *= correction;
                row[i + 1] *= correction;
                row[i + 2] *= correction;
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PreviewMode;
    use crate::ops::{OpScratch, RenderContext};

    fn solid_image(w: usize, h: usize, v: f32) -> LinearImage {
        LinearImage::new(vec![v; w * h * 3], w, h)
    }

    fn ctx() -> OpContext {
        OpContext {
            render: RenderContext {
                wb_coeffs: [1.0; 4],
                cam_to_srgb: crate::color::identity_3x3(),
                is_raw: false,
                capture_sigma: None,
                preview_mode: PreviewMode::None,
                roi: None,
                dcp: None,
            },
            scratch: OpScratch::default(),
        }
    }

    #[test]
    fn amount_zero_is_identity() {
        let mut img = solid_image(64, 48, 0.5);
        let edits = Edits::default();
        LensVignetteOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        for v in &img.rgb {
            if (*v - 0.5).abs() > 1e-6 {
                panic!("expected identity, got {v}");
            }
        }
    }

    #[test]
    fn corrects_darkened_corners() {
        let mut img = solid_image(64, 48, 0.4);
        let edits = Edits {
            lens: LensEdits {
                profile_enabled: Some(true),
                vignette_amount: 100.0,
                vk1: -0.4,
                ..Default::default()
            },
            ..Default::default()
        };
        LensVignetteOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        let center = img.rgb[(24 * 64 + 32) * 3];
        let corner = img.rgb[0];
        if corner <= center + 0.01 {
            panic!("expected corner > center after correction; corner={corner} center={center}");
        }
    }

    #[test]
    fn gain_stays_monotonic_when_polynomial_goes_negative() {
        let (vk1, vk2, vk3) = (-2.3577, 2.7051, -1.2007);
        let mut prev = vignette_correction(vk1, vk2, vk3, 1.0, 0.0);
        for step in 1..=100 {
            let r = step as f32 / 100.0;
            let gain = vignette_correction(vk1, vk2, vk3, 1.0, r);
            if gain < prev - 1e-4 {
                panic!("gain dropped at r={r}: {prev} -> {gain}");
            }
            if gain > VIGNETTE_GAIN_MAX {
                panic!("gain exceeded ceiling at r={r}: {gain}");
            }
            prev = gain;
        }
    }

    #[test]
    fn clamp_keeps_lensfun_vignetting_coefficients() {
        let lens = LensEdits {
            profile_enabled: Some(true),
            vk1: -2.3577,
            vk2: 2.7051,
            vk3: -1.2007,
            ..Default::default()
        }
        .clamped();
        if lens.vk1 != -2.3577 || lens.vk2 != 2.7051 || lens.vk3 != -1.2007 {
            panic!("vignetting coefficients were clamped: {lens:?}");
        }
    }
}
