use std::hash::{Hash, Hasher};
use std::sync::Arc;

use rayon::prelude::*;

use super::DenoisePlane;
use super::model_space::{LumaAxis, ModelSpace};
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::{PipelineError, PipelineResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct DenoiseMix {
    pub(crate) strength: f32,
    pub(crate) detail: f32,
    pub(crate) axis: LumaAxis,
}

impl DenoiseMix {
    fn for_frame(frame: &RawFrame, edits: &Edits) -> PipelineResult<Self> {
        let strength = (edits.detail.ai_denoise_strength / 100.0) as f32;
        let detail = (edits.detail.ai_denoise_detail / 100.0) as f32;
        if detail == 0.0 {
            return Ok(Self {
                strength,
                detail: 0.0,
                axis: LumaAxis::NONE,
            });
        }
        let space = ModelSpace::for_frame(&frame.meta).ok_or_else(|| {
            PipelineError::Render("AI denoise detail needs an invertible camera matrix".into())
        })?;
        Ok(Self {
            strength,
            detail,
            axis: space.luma_axis(),
        })
    }

    fn apply(&self, original: [f32; 3], denoised: [f32; 3]) -> [f32; 3] {
        let residual = self.axis.luma(original) - self.axis.luma(denoised);
        let grain = self.strength * self.detail * residual;
        std::array::from_fn(|c| {
            original[c] + self.strength * (denoised[c] - original[c]) + grain * self.axis.neutral[c]
        })
    }

    pub(crate) fn hash_key(&self, h: &mut impl Hasher) {
        self.strength.to_bits().hash(h);
        self.detail.to_bits().hash(h);
    }
}

pub(crate) fn active_plane<'a>(
    frame: &RawFrame,
    edits: &Edits,
    options: &'a RenderOptions,
) -> PipelineResult<Option<(&'a Arc<DenoisePlane>, DenoiseMix)>> {
    let Some(plane) = options.denoise.as_ref() else {
        return Ok(None);
    };
    if !edits.detail.ai_denoise_active() || !frame.meta.is_raw {
        return Ok(None);
    }
    if plane.width != frame.meta.width || plane.height != frame.meta.height {
        return Err(PipelineError::Render(format!(
            "denoise plane {}x{} does not match the {}x{} frame",
            plane.width, plane.height, frame.meta.width, frame.meta.height
        )));
    }
    Ok(Some((plane, DenoiseMix::for_frame(frame, edits)?)))
}

pub(crate) fn hash_active(plane: Option<&DenoisePlane>, edits: &Edits, h: &mut impl Hasher) {
    plane.map(|plane| plane.key).hash(h);
    if plane.is_none() {
        return;
    }
    let detail = &edits.detail;
    detail.ai_denoise_active().hash(h);
    if detail.ai_denoise_active() {
        detail.ai_denoise_strength.to_bits().hash(h);
        detail.ai_denoise_detail.to_bits().hash(h);
    }
}

pub(crate) fn blend(
    rgb: &mut [f32],
    width: usize,
    plane: &DenoisePlane,
    block: usize,
    mix: DenoiseMix,
) {
    rgb.par_chunks_exact_mut(width * 3)
        .enumerate()
        .for_each(|(y, row)| {
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let denoised = plane.binned_pixel(x, y, block);
                let mixed = mix.apply([px[0], px[1], px[2]], denoised);
                px.copy_from_slice(&mixed);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::super::raw_frame;
    use super::*;
    use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};

    fn plane(width: usize, height: usize, value: f32) -> DenoisePlane {
        DenoisePlane::from_rgb(3, width, height, &vec![value; width * height * 3]).unwrap()
    }

    fn frame() -> RawFrame {
        raw_frame(4, 2, vec![0.0; 4 * 2 * 3])
    }

    fn with_plane(plane: DenoisePlane) -> RenderOptions {
        RenderOptions {
            denoise: Some(Arc::new(plane)),
            ..Default::default()
        }
    }

    fn key(plane: Option<&DenoisePlane>, edits: &Edits) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        hash_active(plane, edits, &mut h);
        h.finish()
    }

    fn mix(strength: f32, detail: f32, axis: LumaAxis) -> DenoiseMix {
        DenoiseMix {
            strength,
            detail,
            axis,
        }
    }

    fn grey_axis() -> LumaAxis {
        LumaAxis {
            weights: [LUMA_R, LUMA_G, LUMA_B],
            neutral: [1.0; 3],
        }
    }

    #[test]
    fn strength_mixes_the_original_toward_the_plane() {
        let mut rgb = vec![0.2f32; 4 * 2 * 3];
        blend(
            &mut rgb,
            4,
            &plane(4, 2, 0.6),
            1,
            mix(0.25, 0.0, LumaAxis::NONE),
        );
        assert!(rgb.iter().all(|v| (v - 0.3).abs() < 1e-4));
        let mut full = vec![0.2f32; 4 * 2 * 3];
        blend(
            &mut full,
            4,
            &plane(4, 2, 0.6),
            1,
            mix(1.0, 0.0, LumaAxis::NONE),
        );
        assert!(full.iter().all(|v| (v - 0.6).abs() < 1e-3));
    }

    #[test]
    fn detail_keeps_luma_noise_and_removes_chroma_noise() {
        let axis = grey_axis();
        let full = mix(1.0, 1.0, axis);
        let clean = [0.3, 0.3, 0.3];
        let bright = full.apply([0.4, 0.4, 0.4], clean);
        assert!(bright.iter().all(|v| (v - 0.4).abs() < 1e-6), "{bright:?}");
        let tinted = full.apply([0.5, 0.3, 0.3], clean);
        assert!((axis.luma(tinted) - axis.luma([0.5, 0.3, 0.3])).abs() < 1e-6);
        assert!((tinted[0] - tinted[1]).abs() < 1e-6, "{tinted:?}");
        assert!((tinted[1] - tinted[2]).abs() < 1e-6, "{tinted:?}");
        let half = mix(1.0, 0.5, axis).apply([0.4, 0.4, 0.4], clean);
        assert!(half.iter().all(|v| (v - 0.35).abs() < 1e-6), "{half:?}");
        let partial = mix(0.6, 0.5, axis).apply([0.4, 0.4, 0.4], clean);
        assert!(
            partial.iter().all(|v| (v - 0.37).abs() < 1e-6),
            "{partial:?}"
        );
        let none = mix(0.0, 1.0, axis).apply([0.5, 0.3, 0.3], clean);
        assert_eq!(none, [0.5, 0.3, 0.3]);
    }

    #[test]
    fn detail_without_an_invertible_matrix_is_an_error() {
        let mut frame = frame();
        frame.meta.wb_coeffs = [1.0, 1.0, 0.0, 0.0];
        let mut edits = Edits::default();
        edits.detail.ai_denoise = true;
        assert_eq!(
            DenoiseMix::for_frame(&frame, &edits).unwrap(),
            mix(0.75, 0.0, LumaAxis::NONE)
        );
        edits.detail.ai_denoise_detail = 40.0;
        assert!(matches!(
            DenoiseMix::for_frame(&frame, &edits),
            Err(PipelineError::Render(_))
        ));
    }

    #[test]
    fn a_superpixel_image_blends_against_the_binned_plane() {
        let mut rgb = vec![0.0f32; 2 * 3];
        blend(
            &mut rgb,
            2,
            &plane(4, 2, 0.5),
            2,
            mix(1.0, 0.0, LumaAxis::NONE),
        );
        assert!(rgb.iter().all(|v| (v - 0.5).abs() < 1e-3));
    }

    #[test]
    fn a_plane_is_only_used_when_enabled_on_a_raw() {
        let raw = frame();
        let options = with_plane(plane(4, 2, 0.5));
        let mut edits = Edits::default();
        assert!(active_plane(&raw, &edits, &options).unwrap().is_none());
        edits.detail.ai_denoise = true;
        assert!(active_plane(&raw, &edits, &options).unwrap().is_some());
        assert!(
            active_plane(&raw, &edits, &RenderOptions::default())
                .unwrap()
                .is_none()
        );
        let mut rendered = frame();
        rendered.meta.is_raw = false;
        assert!(active_plane(&rendered, &edits, &options).unwrap().is_none());
        edits.detail.ai_denoise_strength = 0.0;
        assert!(active_plane(&raw, &edits, &options).unwrap().is_none());
    }

    #[test]
    fn a_plane_that_does_not_match_the_frame_is_an_error() {
        let frame = frame();
        let mut edits = Edits::default();
        edits.detail.ai_denoise = true;
        let wrong = with_plane(plane(2, 2, 0.5));
        assert!(matches!(
            active_plane(&frame, &edits, &wrong),
            Err(PipelineError::Render(_))
        ));
        edits.detail.ai_denoise = false;
        assert!(active_plane(&frame, &edits, &wrong).unwrap().is_none());
    }

    #[test]
    fn the_cache_key_tracks_every_field_only_with_an_active_plane() {
        let plane = plane(4, 2, 0.5);
        let other = DenoisePlane {
            key: plane.key + 1,
            ..plane.clone()
        };
        let off = Edits::default();
        let mut on = off.clone();
        on.detail.ai_denoise = true;
        let mut stronger = on.clone();
        stronger.detail.ai_denoise_strength = 90.0;
        let mut grainier = on.clone();
        grainier.detail.ai_denoise_detail = 30.0;
        let mut off_changed = off.clone();
        off_changed.detail.ai_denoise_strength = 85.0;
        off_changed.detail.ai_denoise_detail = 30.0;
        assert_eq!(key(None, &on), key(None, &off), "no plane");
        assert_ne!(key(Some(&plane), &on), key(None, &on), "plane");
        assert_ne!(key(Some(&plane), &on), key(Some(&plane), &off), "toggle");
        assert_ne!(
            key(Some(&plane), &stronger),
            key(Some(&plane), &on),
            "strength"
        );
        assert_ne!(
            key(Some(&plane), &grainier),
            key(Some(&plane), &on),
            "detail"
        );
        assert_ne!(key(Some(&other), &on), key(Some(&plane), &on), "plane key");
        assert_eq!(
            key(Some(&plane), &off_changed),
            key(Some(&plane), &off),
            "fields while off"
        );
    }
}
