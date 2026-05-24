use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const BINS: usize = 256;

pub(crate) fn chunk_pixels(pixel_count: usize) -> usize {
    let threads = rayon::current_num_threads().max(1);
    let per_thread = pixel_count.div_ceil(threads * 4);
    per_thread.max(4096)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub r: Vec<u32>,
    pub g: Vec<u32>,
    pub b: Vec<u32>,
    pub l: Vec<u32>,
}

impl Histogram {
    pub fn from_rgb(pixels: &[f32], width: usize, height: usize) -> Self {
        let pixel_count = width * height;
        let usable = pixel_count.min(pixels.len() / 3);
        let step = if usable > 500_000 { 2 } else { 1 };
        let chunk = chunk_pixels(usable) * 3;
        let zero = || ([0u32; BINS], [0u32; BINS], [0u32; BINS], [0u32; BINS]);
        let (r, g, b, l) = pixels[..usable * 3]
            .par_chunks(chunk)
            .fold(zero, |mut acc, chunk| {
                let stride = step * 3;
                let mut i = 0;
                while i + 2 < chunk.len() {
                    let rv = (chunk[i].clamp(0.0, 1.0) * 255.0) as usize;
                    let gv = (chunk[i + 1].clamp(0.0, 1.0) * 255.0) as usize;
                    let bv = (chunk[i + 2].clamp(0.0, 1.0) * 255.0) as usize;
                    let lv = (0.2126 * chunk[i] + 0.7152 * chunk[i + 1] + 0.0722 * chunk[i + 2])
                        .clamp(0.0, 1.0);
                    let li = (lv * 255.0) as usize;
                    acc.0[rv.min(BINS - 1)] += 1;
                    acc.1[gv.min(BINS - 1)] += 1;
                    acc.2[bv.min(BINS - 1)] += 1;
                    acc.3[li.min(BINS - 1)] += 1;
                    i += stride;
                }
                acc
            })
            .reduce(zero, |mut a, b| {
                for i in 0..BINS {
                    a.0[i] += b.0[i];
                    a.1[i] += b.1[i];
                    a.2[i] += b.2[i];
                    a.3[i] += b.3[i];
                }
                a
            });
        Self {
            r: r.to_vec(),
            g: g.to_vec(),
            b: b.to_vec(),
            l: l.to_vec(),
        }
    }

    pub fn from_rgb_u8(pixels: &[u8], width: usize, height: usize) -> Self {
        let total_px = width * height;
        let usable = total_px.min(pixels.len() / 3);
        let step = if usable > 500_000 { 2 } else { 1 };
        let chunk = chunk_pixels(usable) * 3;
        let zero = || ([0u32; BINS], [0u32; BINS], [0u32; BINS], [0u32; BINS]);
        let (r, g, b, l) = pixels[..usable * 3]
            .par_chunks(chunk)
            .fold(zero, |mut acc, chunk| {
                let stride = step * 3;
                let mut i = 0;
                while i + 2 < chunk.len() {
                    let rv = chunk[i] as usize;
                    let gv = chunk[i + 1] as usize;
                    let bv = chunk[i + 2] as usize;
                    let li = ((0.2126 * chunk[i] as f32
                        + 0.7152 * chunk[i + 1] as f32
                        + 0.0722 * chunk[i + 2] as f32) as usize)
                        .min(BINS - 1);
                    acc.0[rv.min(BINS - 1)] += 1;
                    acc.1[gv.min(BINS - 1)] += 1;
                    acc.2[bv.min(BINS - 1)] += 1;
                    acc.3[li] += 1;
                    i += stride;
                }
                acc
            })
            .reduce(zero, |mut a, b| {
                for i in 0..BINS {
                    a.0[i] += b.0[i];
                    a.1[i] += b.1[i];
                    a.2[i] += b.2[i];
                    a.3[i] += b.3[i];
                }
                a
            });
        Self {
            r: r.to_vec(),
            g: g.to_vec(),
            b: b.to_vec(),
            l: l.to_vec(),
        }
    }

    pub fn from_rgba8(pixels: &[u8]) -> Self {
        let total_px = pixels.len() / 4;
        let step = if total_px > 500_000 { 2 } else { 1 };
        let chunk = chunk_pixels(total_px) * 4;
        let zero = || ([0u32; BINS], [0u32; BINS], [0u32; BINS], [0u32; BINS]);
        let (r, g, b, l) = pixels
            .par_chunks(chunk)
            .fold(zero, |mut acc, chunk| {
                let stride = step * 4;
                let mut i = 0;
                while i + 3 < chunk.len() {
                    let rv = chunk[i] as usize;
                    let gv = chunk[i + 1] as usize;
                    let bv = chunk[i + 2] as usize;
                    let li = ((0.2126 * chunk[i] as f32
                        + 0.7152 * chunk[i + 1] as f32
                        + 0.0722 * chunk[i + 2] as f32) as usize)
                        .min(BINS - 1);
                    acc.0[rv] += 1;
                    acc.1[gv] += 1;
                    acc.2[bv] += 1;
                    acc.3[li] += 1;
                    i += stride;
                }
                acc
            })
            .reduce(zero, |mut a, b| {
                for i in 0..BINS {
                    a.0[i] += b.0[i];
                    a.1[i] += b.1[i];
                    a.2[i] += b.2[i];
                    a.3[i] += b.3[i];
                }
                a
            });
        Self {
            r: r.to_vec(),
            g: g.to_vec(),
            b: b.to_vec(),
            l: l.to_vec(),
        }
    }

    pub fn pixel_count(&self) -> u64 {
        self.l.iter().map(|&v| v as u64).sum()
    }
}
