use super::LinearImage;
use super::{EditOperator, OpContext, Stage, OpKind};
use crate::PipelineResult;
use crate::edits::{Edits, LensEdits};
use rayon::prelude::*;

pub struct LensVignetteOp;

impl EditOperator for LensVignetteOp {
    fn id(&self) -> &'static str {
        "lens_vignette"
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn kind(&self) -> OpKind {
        OpKind::Spatial
    }
    fn order(&self) -> i32 {
        1
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens.vignette_active()
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
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
}

pub fn vignette_coeffs(lens: &LensEdits) -> (f32, f32, f32) {
    if !lens.profile_enabled {
        return (0.0, 0.0, 0.0);
    }
    let (a, b, c) = lens.effective_vk();
    (a as f32, b as f32, c as f32)
}

#[inline]
pub fn vignette_correction(vk1: f32, vk2: f32, vk3: f32, r_norm: f32) -> f32 {
    let r2 = r_norm * r_norm;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let gain_in = 1.0 + vk1 * r2 + vk2 * r4 + vk3 * r6;
    if gain_in.abs() > 1e-6 {
        1.0 / gain_in
    } else {
        1.0
    }
}

pub fn apply_lens_vignette(image: &mut LinearImage, lens: &LensEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let (vk1, vk2, vk3) = vignette_coeffs(lens);
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
                let correction = vignette_correction(vk1, vk2, vk3, r);
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

    fn solid_image(w: usize, h: usize, v: f32) -> LinearImage {
        LinearImage::new(vec![v; w * h * 3], w, h)
    }

    fn ctx() -> OpContext {
        OpContext {
            wb_coeffs: [1.0; 4],
            cam_to_srgb: crate::color::identity_3x3(),
            is_raw: false,
            preview_mode: PreviewMode::None,
            shadows_blur: None,
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
                profile_enabled: true,
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
}
