use rayon::prelude::*;

use crate::ops::LinearImage;

pub const SELECTOR_EPS: f32 = 0.0016;
const SELECTOR_SPAN: f32 = 1024.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectorTaps {
    pub whole: u32,
    pub frac: f32,
}

impl SelectorTaps {
    pub fn for_long_edge(long_edge: f32) -> Self {
        let radius = long_edge / SELECTOR_SPAN;
        Self {
            whole: radius.floor() as u32,
            frac: radius.fract(),
        }
    }

    pub fn reach(self) -> u32 {
        self.whole + 1
    }

    fn mean(self, sample: impl Fn(i64) -> [f32; 2]) -> [f32; 2] {
        let reach = self.reach() as i64;
        let norm = 2.0 * (self.whole as f32 + self.frac) + 1.0;
        let total = (-reach..=reach).fold([0.0f32; 2], |acc, k| {
            let weight = if k.unsigned_abs() <= self.whole as u64 {
                1.0
            } else {
                self.frac
            };
            let s = sample(k);
            [acc[0] + weight * s[0], acc[1] + weight * s[1]]
        });
        [total[0] / norm, total[1] / norm]
    }
}

pub(super) fn display_selector(image: &LinearImage) -> Vec<[f32; 3]> {
    let display: Vec<[f32; 3]> = image
        .rgb
        .par_chunks_exact(3)
        .map(|p| crate::tone::apply_rgb([p[0], p[1], p[2]]))
        .collect();
    smooth_selector(display, image.width, image.height)
}

pub(super) fn smooth_selector(
    mut display: Vec<[f32; 3]>,
    width: usize,
    height: usize,
) -> Vec<[f32; 3]> {
    let taps = SelectorTaps::for_long_edge(width.max(height) as f32);
    for channel in 0..3 {
        let moments: Vec<[f32; 2]> = display
            .par_iter()
            .map(|p| [p[channel], p[channel] * p[channel]])
            .collect();
        let coeffs: Vec<[f32; 2]> = box_mean(moments, width, height, taps)
            .into_par_iter()
            .map(|[mean, mean_sq]| {
                let variance = (mean_sq - mean * mean).max(0.0);
                let gain = variance / (variance + SELECTOR_EPS);
                [gain, mean - gain * mean]
            })
            .collect();
        let coeffs = box_mean(coeffs, width, height, taps);
        display
            .par_iter_mut()
            .zip(coeffs.par_iter())
            .for_each(|(p, [gain, offset])| p[channel] = gain * p[channel] + offset);
    }
    display
}

fn box_mean(
    mut values: Vec<[f32; 2]>,
    width: usize,
    height: usize,
    taps: SelectorTaps,
) -> Vec<[f32; 2]> {
    let clamp = |i: usize, k: i64, n: usize| (i as i64 + k).clamp(0, n as i64 - 1) as usize;
    values.par_chunks_exact_mut(width).for_each(|row| {
        let line = row.to_vec();
        row.iter_mut()
            .enumerate()
            .for_each(|(x, out)| *out = taps.mean(|k| line[clamp(x, k, width)]));
    });
    let mut out = vec![[0.0f32; 2]; values.len()];
    out.par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(y, row)| {
            row.iter_mut().enumerate().for_each(|(x, o)| {
                *o = taps.mean(|k| values[clamp(y, k, height) * width + x]);
            })
        });
    out
}
