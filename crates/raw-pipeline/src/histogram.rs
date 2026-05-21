use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const BINS: usize = 256;

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
        let zero = || ([0u32; BINS], [0u32; BINS], [0u32; BINS], [0u32; BINS]);
        let (r, g, b, l) = pixels[..usable * 3]
            .par_chunks(30_000 * 3)
            .fold(zero, |mut acc, chunk| {
                for px in chunk.chunks_exact(3) {
                    let rv = (px[0].clamp(0.0, 1.0) * 255.0) as usize;
                    let gv = (px[1].clamp(0.0, 1.0) * 255.0) as usize;
                    let bv = (px[2].clamp(0.0, 1.0) * 255.0) as usize;
                    let lv = (0.2126 * px[0] + 0.7152 * px[1] + 0.0722 * px[2]).clamp(0.0, 1.0);
                    let li = (lv * 255.0) as usize;
                    acc.0[rv.min(BINS - 1)] += 1;
                    acc.1[gv.min(BINS - 1)] += 1;
                    acc.2[bv.min(BINS - 1)] += 1;
                    acc.3[li.min(BINS - 1)] += 1;
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
        let mut r = vec![0u32; BINS];
        let mut g = vec![0u32; BINS];
        let mut b = vec![0u32; BINS];
        let mut l = vec![0u32; BINS];
        for px in pixels.chunks_exact(4) {
            let rv = px[0] as usize;
            let gv = px[1] as usize;
            let bv = px[2] as usize;
            let li = ((0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32)
                as usize)
                .min(BINS - 1);
            r[rv] += 1;
            g[gv] += 1;
            b[bv] += 1;
            l[li] += 1;
        }
        Self { r, g, b, l }
    }

    pub fn pixel_count(&self) -> u64 {
        self.l.iter().map(|&v| v as u64).sum()
    }
}
