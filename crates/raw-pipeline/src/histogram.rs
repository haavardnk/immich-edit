use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const BINS: usize = 256;
const SUBSAMPLE_ABOVE: usize = 500_000;
const LUMA_WEIGHTS: [u32; 3] = [2126, 7152, 722];
pub(crate) const LUMA_SCALE: u32 = 10_000;

pub(crate) fn chunk_pixels(pixel_count: usize) -> usize {
    let threads = rayon::current_num_threads().max(1);
    let per_thread = pixel_count.div_ceil(threads * 4);
    per_thread.max(4096)
}

pub(crate) fn sample_step(pixel_count: usize) -> usize {
    if pixel_count > SUBSAMPLE_ABOVE { 2 } else { 1 }
}

pub(crate) fn weighted_luma(r: u8, g: u8, b: u8) -> u32 {
    LUMA_WEIGHTS[0] * r as u32 + LUMA_WEIGHTS[1] * g as u32 + LUMA_WEIGHTS[2] * b as u32
}

pub(crate) fn display_luma(r: u8, g: u8, b: u8) -> usize {
    (weighted_luma(r, g, b) / LUMA_SCALE) as usize
}

#[cfg(feature = "native")]
fn linear_bin(v: f32) -> usize {
    ((v.clamp(0.0, 1.0) * 255.0) as usize).min(BINS - 1)
}

#[derive(Clone)]
pub(crate) struct Bins {
    r: [u32; BINS],
    g: [u32; BINS],
    b: [u32; BINS],
    l: [u32; BINS],
}

impl Bins {
    pub(crate) fn zero() -> Self {
        Self {
            r: [0; BINS],
            g: [0; BINS],
            b: [0; BINS],
            l: [0; BINS],
        }
    }

    pub(crate) fn add_display(&mut self, r: u8, g: u8, b: u8) {
        self.r[r as usize] += 1;
        self.g[g as usize] += 1;
        self.b[b as usize] += 1;
        self.l[display_luma(r, g, b)] += 1;
    }

    #[cfg(feature = "native")]
    pub(crate) fn add_linear(&mut self, r: f32, g: f32, b: f32) {
        self.r[linear_bin(r)] += 1;
        self.g[linear_bin(g)] += 1;
        self.b[linear_bin(b)] += 1;
        self.l[linear_bin(crate::math::luma(r, g, b))] += 1;
    }

    pub(crate) fn merge(mut self, other: Self) -> Self {
        let pairs = [
            (&mut self.r, &other.r),
            (&mut self.g, &other.g),
            (&mut self.b, &other.b),
            (&mut self.l, &other.l),
        ];
        for (into, from) in pairs {
            into.iter_mut().zip(from).for_each(|(a, b)| *a += b);
        }
        self
    }

    pub(crate) fn into_histogram(self) -> Histogram {
        Histogram {
            r: self.r.to_vec(),
            g: self.g.to_vec(),
            b: self.b.to_vec(),
            l: self.l.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Histogram {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    pub l: Vec<u32>,
}

impl Histogram {
    pub fn from_rgb_u8(pixels: &[u8], width: usize, height: usize) -> Self {
        let count = (width * height).min(pixels.len() / 3);
        let step = sample_step(count);
        (0..count)
            .into_par_iter()
            .step_by(step)
            .with_min_len(chunk_pixels(count) / step)
            .fold(Bins::zero, |mut bins, i| {
                bins.add_display(pixels[i * 3], pixels[i * 3 + 1], pixels[i * 3 + 2]);
                bins
            })
            .reduce(Bins::zero, Bins::merge)
            .into_histogram()
    }

    pub(crate) fn from_counts(counts: &[u32]) -> Self {
        let channel = |c: usize| counts[c * BINS..(c + 1) * BINS].to_vec();
        Self {
            r: channel(0),
            g: channel(1),
            b: channel(2),
            l: channel(3),
        }
    }

    pub fn pixel_count(&self) -> u64 {
        self.l.iter().map(|&v| v as u64).sum()
    }
}

#[cfg(test)]
mod tests;
