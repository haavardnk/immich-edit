use crate::color::{inverse_3x3, mat3_vec};
use crate::dcp::setup::matrix_white;
use crate::frame::FrameMeta;
use crate::math::{luma, srgb_to_linear};
use crate::ops::white_balance::camera_wb;
use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};
use crate::tone::srgb_oetf_scalar;

const EXPOSURE_TARGET: f32 = 0.18;
const MAX_EXPOSURE_GAIN: f32 = 16.0;
const EXPOSURE_SAMPLE_STRIDE: usize = 7;

type Mat3 = [[f32; 3]; 3];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelSpace {
    forward: Mat3,
    inverse: Mat3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LumaAxis {
    pub(crate) weights: [f32; 3],
    pub(crate) neutral: [f32; 3],
}

impl LumaAxis {
    pub(crate) const NONE: Self = Self {
        weights: [0.0; 3],
        neutral: [0.0; 3],
    };

    pub(crate) fn luma(&self, cam: [f32; 3]) -> f32 {
        cam.iter().zip(self.weights).map(|(v, w)| v * w).sum()
    }
}

impl ModelSpace {
    fn from_forward(forward: Mat3) -> Option<Self> {
        let inverse = inverse_3x3(forward)?;
        Some(Self { forward, inverse })
    }

    pub fn for_frame(meta: &FrameMeta) -> Option<Self> {
        let wb = camera_wb(meta.wb_coeffs);
        let cam_to_srgb = matrix_white(meta).cam_to_srgb();
        Self::from_forward(cam_to_srgb.map(|row| std::array::from_fn(|c| row[c] * wb[c])))
    }

    pub(crate) fn luma_axis(&self) -> LumaAxis {
        let [r, g, b] = self.forward;
        let weights = std::array::from_fn(|c| LUMA_R * r[c] + LUMA_G * g[c] + LUMA_B * b[c]);
        let neutral = mat3_vec(&self.inverse, [1.0; 3]);
        LumaAxis { weights, neutral }
    }

    pub fn exposed_for(self, cam: &[f32]) -> Option<Self> {
        let mut lumas: Vec<f32> = cam
            .as_chunks::<3>()
            .0
            .iter()
            .step_by(EXPOSURE_SAMPLE_STRIDE)
            .map(|px| {
                let [r, g, b] = mat3_vec(&self.forward, [px[0], px[1], px[2]]);
                luma(r, g, b)
            })
            .filter(|luma| luma.is_finite())
            .collect();
        if lumas.is_empty() {
            return Some(self);
        }
        let middle = lumas.len() / 2;
        let (_, median, _) = lumas.select_nth_unstable_by(middle, f32::total_cmp);
        let gain = (EXPOSURE_TARGET / median.max(f32::MIN_POSITIVE)).clamp(1.0, MAX_EXPOSURE_GAIN);
        let forward = self.forward.map(|row| row.map(|v| v * gain));
        Self::from_forward(forward)
    }

    pub fn linear_input(&self, cam: [f32; 3]) -> [f32; 3] {
        mat3_vec(&self.forward, cam).map(|v| v.clamp(0.0, 1.0))
    }

    pub fn encode(&self, cam: [f32; 3]) -> [f32; 3] {
        self.linear_input(cam).map(srgb_oetf_scalar)
    }

    pub fn restore(&self, cam: [f32; 3], model_out: [f32; 3]) -> [f32; 3] {
        let before = self.linear_input(cam);
        let after = model_out.map(srgb_to_linear);
        let delta = std::array::from_fn(|c| after[c] - before[c]);
        let back = mat3_vec(&self.inverse, delta);
        std::array::from_fn(|c| cam[c] + back[c])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forward() -> Mat3 {
        [[3.2, -0.5, -0.15], [-0.4, 1.4, -0.3], [0.0, -0.4, 2.1]]
    }

    fn space() -> ModelSpace {
        ModelSpace::from_forward(forward()).unwrap()
    }

    #[test]
    fn an_unchanged_model_output_restores_the_camera_values() {
        let space = space();
        for cam in [[0.05, 0.1, 0.07], [0.2, 0.3, 0.25], [0.01, 0.0, 0.02]] {
            let restored = space.restore(cam, space.encode(cam));
            for c in 0..3 {
                assert!(
                    (restored[c] - cam[c]).abs() < 1e-4,
                    "{cam:?} -> {restored:?}"
                );
            }
        }
    }

    #[test]
    fn clipped_highlights_keep_their_unclamped_values() {
        let space = space();
        let hot = [3.0, 4.0, 3.5];
        let restored = space.restore(hot, space.encode(hot));
        assert_eq!(restored, hot);
    }

    #[test]
    fn a_model_change_maps_back_through_the_inverse() {
        let space = space();
        let cam = [0.1, 0.2, 0.15];
        let shifted = space.encode(cam).map(|v| (v + 0.02).min(1.0));
        let restored = space.restore(cam, shifted);
        let lin_before = space.linear_input(cam);
        let lin_after = space.linear_input(restored);
        let expected = shifted.map(srgb_to_linear);
        for c in 0..3 {
            assert!((lin_after[c] - expected[c]).abs() < 1e-4);
            assert!(lin_after[c] > lin_before[c]);
        }
    }

    #[test]
    fn a_dark_frame_is_lifted_to_mid_grey_and_still_round_trips() {
        let dark = [0.01f32, 0.012, 0.011];
        let cam: Vec<f32> = (0..64).flat_map(|_| dark).collect();
        let base = space();
        let lifted = base.exposed_for(&cam).unwrap();
        let [r, g, b] = lifted.linear_input(dark);
        let lifted_luma = luma(r, g, b);
        assert!(
            (lifted_luma - EXPOSURE_TARGET).abs() < 1e-3,
            "{lifted_luma}"
        );
        let restored = lifted.restore(dark, lifted.encode(dark));
        for c in 0..3 {
            assert!((restored[c] - dark[c]).abs() < 1e-5);
        }
        let bright: Vec<f32> = (0..64).flat_map(|_| [0.5f32, 0.6, 0.55]).collect();
        assert_eq!(base.exposed_for(&bright).unwrap(), base);
        let black = vec![0.0f32; 192];
        let crushed = base.exposed_for(&black).unwrap();
        assert_eq!(
            crushed.forward,
            base.forward.map(|row| row.map(|v| v * MAX_EXPOSURE_GAIN))
        );
    }

    #[test]
    fn a_singular_matrix_has_no_model_space() {
        let singular = [[1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]];
        assert!(ModelSpace::from_forward(singular).is_none());
    }

    #[test]
    fn the_neutral_axis_is_display_grey_of_unit_luma() {
        let axis = space().luma_axis();
        let [r, g, b] = mat3_vec(&forward(), axis.neutral);
        for v in [r, g, b] {
            assert!((v - 1.0).abs() < 1e-5, "{r} {g} {b}");
        }
        assert!((axis.luma(axis.neutral) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn camera_luma_matches_display_luma() {
        let axis = space().luma_axis();
        let cam = [0.1, 0.25, 0.07];
        let [r, g, b] = mat3_vec(&forward(), cam);
        assert!((axis.luma(cam) - luma(r, g, b)).abs() < 1e-6);
    }
}
