use super::LinearImage;
use super::sample::sample_rgb_bicubic;
use super::{EditOperator, OpContext, Stage};
use crate::PipelineResult;
use crate::edits::{Edits, LensEdits};
use rayon::prelude::*;

pub struct LensDistortionOp;

impl EditOperator for LensDistortionOp {
    fn id(&self) -> &'static str {
        "lens_distortion"
    }
    fn stage(&self) -> Stage {
        Stage::Sensor
    }
    fn order(&self) -> i32 {
        0
    }
    fn is_active(&self, edits: &Edits) -> bool {
        edits.lens.distortion_active()
    }
    fn apply_cpu(
        &self,
        image: &mut LinearImage,
        _ctx: &OpContext,
        edits: &Edits,
    ) -> PipelineResult<()> {
        apply_lens_distortion(image, &edits.lens);
        Ok(())
    }
    fn to_doc(&self, _edits: &Edits) -> Option<serde_json::Value> {
        None
    }
}

pub fn distortion_coeffs(lens: &LensEdits) -> (f32, f32, f32) {
    if !lens.profile_enabled {
        return (0.0, 0.0, 0.0);
    }
    let (k1, k2, k3) = lens.effective_k();
    (k1 as f32, k2 as f32, k3 as f32)
}

pub fn constrain_zoom(k1: f32, k2: f32, k3: f32) -> f32 {
    let s = |r: f32| {
        let r2 = r * r;
        1.0 + k1 * r2 + k2 * r2 * r2 + k3 * r2 * r2 * r2
    };
    if s(1.0) <= 1.0 {
        return 1.0;
    }
    let mut z: f32 = 1.0;
    for _ in 0..32 {
        let sz = s(z);
        if sz <= 1.0 {
            return 1.0;
        }
        let next = 1.0 / sz;
        if (next - z).abs() < 1e-6 {
            return next;
        }
        z = next;
    }
    z
}

pub fn distortion_zoom(lens: &LensEdits) -> f32 {
    if !lens.profile_enabled || !lens.constrain_crop {
        return 1.0;
    }
    let (k1, k2, k3) = distortion_coeffs(lens);
    constrain_zoom(k1, k2, k3)
}

pub fn apply_lens_distortion(image: &mut LinearImage, lens: &LensEdits) {
    let w = image.width;
    let h = image.height;
    if w == 0 || h == 0 {
        return;
    }
    let (k1, k2, k3) = distortion_coeffs(lens);
    let zoom = distortion_zoom(lens);
    let cx = w as f32 * 0.5;
    let cy = h as f32 * 0.5;
    let r_norm = 0.5 * ((w as f32).powi(2) + (h as f32).powi(2)).sqrt();
    let inv_norm = zoom / r_norm;
    let src = image.rgb.clone();
    image
        .rgb
        .par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, row)| {
            let dy = (y as f32 + 0.5 - cy) * inv_norm;
            for x in 0..w {
                let dx = (x as f32 + 0.5 - cx) * inv_norm;
                let r2 = dx * dx + dy * dy;
                let r4 = r2 * r2;
                let r6 = r4 * r2;
                let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
                let sx = dx * s * r_norm + cx - 0.5;
                let sy = dy * s * r_norm + cy - 0.5;
                let sample = sample_rgb_bicubic(&src, w, h, sx, sy);
                let i = x * 3;
                row[i] = sample[0];
                row[i + 1] = sample[1];
                row[i + 2] = sample[2];
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::PreviewMode;

    #[test]
    fn constrain_zoom_solves_fixed_point() {
        let k1 = 0.18;
        let z = constrain_zoom(k1, 0.0, 0.0);
        let s = 1.0 + k1 * z * z;
        if (z * s - 1.0).abs() > 1e-4 {
            panic!("z*s(z) should equal 1, got {}", z * s);
        }
        if z >= 1.0 {
            panic!("barrel should require zoom < 1, got {z}");
        }
    }

    #[test]
    fn constrain_zoom_noop_for_pincushion() {
        let z = constrain_zoom(-0.1, 0.0, 0.0);
        if (z - 1.0).abs() > 1e-6 {
            panic!("pincushion should not crop, got {z}");
        }
    }

    fn gradient_image(w: usize, h: usize) -> LinearImage {
        let mut rgb = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) * 3;
                let v = (x as f32) / (w as f32 - 1.0);
                rgb[i] = v;
                rgb[i + 1] = v;
                rgb[i + 2] = v;
            }
        }
        LinearImage::new(rgb, w, h)
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
    fn inactive_is_identity() {
        let mut img = gradient_image(32, 24);
        let before = img.rgb.clone();
        let edits = Edits {
            lens: LensEdits {
                profile_enabled: true,
                distortion_amount: 50.0,
                ..Default::default()
            },
            ..Default::default()
        };
        LensDistortionOp
            .apply_cpu(&mut img, &ctx(), &edits)
            .unwrap();
        for (a, b) in img.rgb.iter().zip(before.iter()) {
            if (a - b).abs() > 1e-4 {
                panic!("inactive (k=0) should not modify image; got {a} vs {b}");
            }
        }
    }

    #[test]
    fn barrel_then_pincushion_round_trip_close() {
        let mut img = gradient_image(64, 48);
        let target = img.rgb.clone();
        let barrel = Edits {
            lens: LensEdits {
                profile_enabled: true,
                distortion_amount: 100.0,
                k1: -0.1,
                ..Default::default()
            },
            ..Default::default()
        };
        let pincushion = Edits {
            lens: LensEdits {
                profile_enabled: true,
                distortion_amount: 100.0,
                k1: 0.1,
                ..Default::default()
            },
            ..Default::default()
        };
        LensDistortionOp
            .apply_cpu(&mut img, &ctx(), &barrel)
            .unwrap();
        if img.rgb == target {
            panic!("barrel should modify image");
        }
        LensDistortionOp
            .apply_cpu(&mut img, &ctx(), &pincushion)
            .unwrap();
        let center_idx = (24 * 64 + 32) * 3;
        if (img.rgb[center_idx] - target[center_idx]).abs() > 0.05 {
            panic!(
                "round trip too far at center: got {}, expected {}",
                img.rgb[center_idx], target[center_idx]
            );
        }
    }
}
