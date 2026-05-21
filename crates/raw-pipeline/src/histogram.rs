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
        let mut r = vec![0u32; BINS];
        let mut g = vec![0u32; BINS];
        let mut b = vec![0u32; BINS];
        let mut l = vec![0u32; BINS];

        let pixel_count = width * height;
        for i in 0..pixel_count {
            let idx = i * 3;
            if idx + 2 >= pixels.len() {
                break;
            }
            let rv = (pixels[idx].clamp(0.0, 1.0) * 255.0) as usize;
            let gv = (pixels[idx + 1].clamp(0.0, 1.0) * 255.0) as usize;
            let bv = (pixels[idx + 2].clamp(0.0, 1.0) * 255.0) as usize;
            let lv = (0.2126 * pixels[idx] + 0.7152 * pixels[idx + 1] + 0.0722 * pixels[idx + 2])
                .clamp(0.0, 1.0);
            let li = (lv * 255.0) as usize;

            r[rv.min(BINS - 1)] += 1;
            g[gv.min(BINS - 1)] += 1;
            b[bv.min(BINS - 1)] += 1;
            l[li.min(BINS - 1)] += 1;
        }

        Self { r, g, b, l }
    }

    pub fn pixel_count(&self) -> u64 {
        self.l.iter().map(|&v| v as u64).sum()
    }
}
