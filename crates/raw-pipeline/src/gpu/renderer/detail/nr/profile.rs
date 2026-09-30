use crate::ops::denoise::estimate::{HIST_LEN, NoiseCurve, fit};
use crate::ops::denoise::{CHROMA_LEVELS, LUMA_LEVELS};

pub(super) const HIST_BLOCKS: usize = LUMA_LEVELS + 2 * CHROMA_LEVELS + 2;
pub(super) const FINE_BLOCK: usize = LUMA_LEVELS + 2 * CHROMA_LEVELS;

pub(super) fn luma_block(level: usize) -> usize {
    level
}

pub(super) fn chroma_block(level: usize) -> usize {
    LUMA_LEVELS + 2 * level
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::gpu::renderer) struct NoiseProfile {
    pub luma: [NoiseCurve; LUMA_LEVELS],
    pub chroma: [[NoiseCurve; 2]; CHROMA_LEVELS],
    pub fine: [NoiseCurve; 2],
}

impl NoiseProfile {
    pub fn from_histograms(hist: &[u32]) -> Self {
        let block = |b: usize| &hist[b * HIST_LEN..(b + 1) * HIST_LEN];
        let mut coarser_than = None;
        let luma = std::array::from_fn(|s| {
            let curve = fit(block(luma_block(s)), coarser_than);
            coarser_than = Some(curve);
            curve
        });
        let mut coarser_than = [None; 2];
        let chroma = std::array::from_fn(|s| {
            let curves: [NoiseCurve; 2] =
                std::array::from_fn(|c| fit(block(chroma_block(s) + c), coarser_than[c]));
            coarser_than = curves.map(Some);
            curves
        });
        let fine = std::array::from_fn(|c| fit(block(FINE_BLOCK + c), None));
        Self { luma, chroma, fine }
    }
}

pub(super) fn packed(curves: &[NoiseCurve]) -> [f32; 4] {
    std::array::from_fn(|i| {
        curves
            .get(i / 2)
            .map_or(0.0, |c| if i % 2 == 0 { c.a } else { c.b })
    })
}
