use std::borrow::Cow;

use rayon::prelude::*;

use super::DenoisePlane;
use super::low_frequency::restore_low_frequencies;
use super::model_space::ModelSpace;
use super::tiles::{Tiling, run_tiled};
use crate::cpu::pipeline::full_demosaic;
use crate::frame::RawFrame;
use crate::{PipelineError, PipelineResult};

const TILE_MARGIN: usize = 16;
const TILE_OVERLAP: usize = 48;

fn tiling(width: usize, height: usize, edge: usize) -> Tiling {
    let margin = TILE_MARGIN.min(edge / 8);
    let overlap = TILE_OVERLAP.min((edge - 2 * margin) / 4);
    Tiling {
        width,
        height,
        edge,
        margin,
        overlap,
    }
}

pub fn bake<F, P>(
    frame: &RawFrame,
    key: u64,
    edge: usize,
    run: F,
    progress: P,
) -> PipelineResult<DenoisePlane>
where
    F: FnMut(&[f32]) -> PipelineResult<Vec<f32>>,
    P: FnMut(usize, usize),
{
    if !frame.meta.is_raw {
        return Err(PipelineError::Unsupported(
            "AI denoise needs a RAW photo".into(),
        ));
    }
    let width = frame.meta.width;
    let height = frame.meta.height;
    let no_matrix =
        || PipelineError::Unsupported("AI denoise needs an invertible camera matrix".into());
    let space = ModelSpace::for_frame(&frame.meta).ok_or_else(no_matrix)?;
    let cam: Cow<[f32]> = if frame.cpp == 3 {
        Cow::Borrowed(&frame.data)
    } else {
        Cow::Owned(full_demosaic(frame))
    };
    let space = space.exposed_for(&cam).ok_or_else(no_matrix)?;
    let input = |i: usize| [cam[i * 3], cam[i * 3 + 1], cam[i * 3 + 2]];
    let mut denoised = run_tiled(
        tiling(width, height, edge),
        |i| space.encode(input(i)),
        run,
        progress,
    )?;
    denoised
        .par_chunks_exact_mut(3)
        .enumerate()
        .for_each(|(i, px)| {
            let restored = space.restore(input(i), [px[0], px[1], px[2]]);
            px.copy_from_slice(&restored);
        });
    restore_low_frequencies(&cam, &mut denoised, width);
    DenoisePlane::from_rgb(key, width, height, &denoised)
}

#[cfg(test)]
mod tests {
    use super::super::raw_frame;
    use super::*;

    fn frame(is_raw: bool) -> RawFrame {
        let width = 40;
        let height = 24;
        let data: Vec<f32> = (0..width * height)
            .flat_map(|i| {
                let x = (i % width) as f32 / width as f32;
                [0.05 + 0.1 * x, 0.1 + 0.05 * x, 0.08]
            })
            .collect();
        let mut frame = raw_frame(width, height, data);
        frame.meta.wb_coeffs = [2.0, 1.0, 1.5, 0.0];
        frame.meta.xyz_to_cam = [
            [0.7, -0.1, -0.05],
            [-0.4, 1.2, 0.2],
            [-0.05, 0.15, 0.6],
            [0.0, 0.0, 0.0],
        ];
        frame.meta.is_raw = is_raw;
        frame
    }

    #[test]
    fn an_identity_model_bakes_the_frame_back() {
        let frame = frame(true);
        let plane = bake(&frame, 7, 16, |t| Ok(t.to_vec()), |_, _| {}).unwrap();
        assert_eq!((plane.key, plane.width, plane.height), (7, 40, 24));
        let worst = plane
            .rgb_f16
            .iter()
            .zip(&frame.data)
            .map(|(bits, v)| (half::f16::from_bits(*bits).to_f32() - v).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 2e-3, "{worst}");
    }

    #[test]
    fn a_rendered_photo_is_not_baked() {
        let result = bake(&frame(false), 1, 16, |t| Ok(t.to_vec()), |_, _| {});
        assert!(matches!(result, Err(PipelineError::Unsupported(_))));
    }
}
