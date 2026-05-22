use crate::ops::LinearImage;
use rayon::prelude::*;

pub struct LumaPyramid {
    pub levels: Vec<Vec<f32>>,
    pub dims: Vec<(usize, usize)>,
}

impl LumaPyramid {
    pub fn build(image: &LinearImage, num_levels: usize) -> Self {
        let n = num_levels.max(1);
        let mut levels: Vec<Vec<f32>> = Vec::with_capacity(n);
        let mut dims: Vec<(usize, usize)> = Vec::with_capacity(n);
        let l0: Vec<f32> = image
            .rgb
            .par_chunks_exact(3)
            .map(|p| 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2])
            .collect();
        levels.push(l0);
        dims.push((image.width, image.height));
        for _ in 1..n {
            let (pw, ph) = *dims.last().unwrap();
            let nw = (pw / 2).max(1);
            let nh = (ph / 2).max(1);
            let prev = levels.last().unwrap().clone();
            let mut next = vec![0.0f32; nw * nh];
            next.par_chunks_exact_mut(nw)
                .enumerate()
                .for_each(|(y, row)| {
                    for (x, slot) in row.iter_mut().enumerate() {
                        let sx = x * 2;
                        let sy = y * 2;
                        let a = prev[sy * pw + sx];
                        let b = if sx + 1 < pw {
                            prev[sy * pw + sx + 1]
                        } else {
                            a
                        };
                        let c = if sy + 1 < ph {
                            prev[(sy + 1) * pw + sx]
                        } else {
                            a
                        };
                        let d = if sx + 1 < pw && sy + 1 < ph {
                            prev[(sy + 1) * pw + sx + 1]
                        } else {
                            a
                        };
                        *slot = (a + b + c + d) * 0.25;
                    }
                });
            levels.push(next);
            dims.push((nw, nh));
        }
        Self { levels, dims }
    }

    pub fn sample(&self, level: u32, fx: f32, fy: f32) -> f32 {
        let level = (level as usize).min(self.levels.len() - 1);
        let scale = 1.0 / (1u32 << level) as f32;
        let lx = fx * scale - 0.5;
        let ly = fy * scale - 0.5;
        let x0 = lx.floor() as i32;
        let y0 = ly.floor() as i32;
        let tx = lx - x0 as f32;
        let ty = ly - y0 as f32;
        let (w, h) = self.dims[level];
        let buf = &self.levels[level];
        let load = |x: i32, y: i32| -> f32 {
            let cx = x.clamp(0, w as i32 - 1) as usize;
            let cy = y.clamp(0, h as i32 - 1) as usize;
            buf[cy * w + cx]
        };
        let l00 = load(x0, y0);
        let l10 = load(x0 + 1, y0);
        let l01 = load(x0, y0 + 1);
        let l11 = load(x0 + 1, y0 + 1);
        let lx0 = l00 + (l10 - l00) * tx;
        let lx1 = l01 + (l11 - l01) * tx;
        lx0 + (lx1 - lx0) * ty
    }
}
