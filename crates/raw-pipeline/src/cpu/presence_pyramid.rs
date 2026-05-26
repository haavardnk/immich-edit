use crate::cpu::scratch::Scratch;
use crate::ops::LinearImage;
use rayon::prelude::*;

#[derive(Debug)]
pub struct LumaPyramid {
    pub levels: Vec<Scratch>,
    pub dims: Vec<(usize, usize)>,
}

impl LumaPyramid {
    pub fn build(image: &LinearImage, num_levels: usize) -> Self {
        let n = num_levels.max(1);
        let mut levels: Vec<Scratch> = Vec::with_capacity(n);
        let mut dims: Vec<(usize, usize)> = Vec::with_capacity(n);
        let n0 = image.width * image.height;
        let mut l0 = Scratch::take_uninit(n0);
        l0.par_iter_mut()
            .zip(image.rgb.par_chunks_exact(3))
            .for_each(|(slot, p)| {
                *slot = 0.2126 * p[0] + 0.7152 * p[1] + 0.0722 * p[2];
            });
        levels.push(l0);
        dims.push((image.width, image.height));
        for _ in 1..n {
            let last_idx = levels.len() - 1;
            let (pw, ph) = dims[last_idx];
            let nw = (pw / 2).max(1);
            let nh = (ph / 2).max(1);
            let mut next = Scratch::take_uninit(nw * nh);
            {
                let prev = &levels[last_idx];
                let interior_w = if pw >= 2 { nw.min(pw / 2) } else { 0 };
                next.par_chunks_exact_mut(nw)
                    .enumerate()
                    .for_each(|(y, row)| {
                        let sy = y * 2;
                        let bottom_ok = sy + 1 < ph;
                        let r0 = &prev[sy * pw..sy * pw + pw];
                        let r1 = if bottom_ok {
                            &prev[(sy + 1) * pw..(sy + 1) * pw + pw]
                        } else {
                            r0
                        };
                        for (x, slot) in row[..interior_w].iter_mut().enumerate() {
                            let sx = x * 2;
                            *slot = (r0[sx] + r0[sx + 1] + r1[sx] + r1[sx + 1]) * 0.25;
                        }
                        if interior_w < nw {
                            let sx = interior_w * 2;
                            let a = r0[sx];
                            let b = if sx + 1 < pw { r0[sx + 1] } else { a };
                            let c = r1[sx];
                            let d = if sx + 1 < pw { r1[sx + 1] } else { c };
                            row[interior_w] = (a + b + c + d) * 0.25;
                        }
                    });
            }
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

    pub fn upsample(&self, level: u32, w: usize, h: usize) -> Vec<f32> {
        let level = (level as usize).min(self.levels.len() - 1);
        let scale = 1.0 / (1u32 << level) as f32;
        let (mw, mh) = self.dims[level];
        let mip = &self.levels[level];
        let mw_i = mw as i32;
        let mh_i = mh as i32;
        let mut out = Scratch::take_uninit(w * h);
        out.par_chunks_exact_mut(w)
            .enumerate()
            .for_each(|(y, row)| {
                let ly = (y as f32 + 0.5) * scale - 0.5;
                let y0 = ly.floor() as i32;
                let ty = ly - y0 as f32;
                let ya = y0.clamp(0, mh_i - 1) as usize;
                let yb = (y0 + 1).clamp(0, mh_i - 1) as usize;
                let ra = ya * mw;
                let rb = yb * mw;
                for (x, slot) in row.iter_mut().enumerate() {
                    let lx = (x as f32 + 0.5) * scale - 0.5;
                    let x0 = lx.floor() as i32;
                    let tx = lx - x0 as f32;
                    let xa = x0.clamp(0, mw_i - 1) as usize;
                    let xb = (x0 + 1).clamp(0, mw_i - 1) as usize;
                    let l00 = mip[ra + xa];
                    let l10 = mip[ra + xb];
                    let l01 = mip[rb + xa];
                    let l11 = mip[rb + xb];
                    let lx0 = l00 + (l10 - l00) * tx;
                    let lx1 = l01 + (l11 - l01) * tx;
                    *slot = lx0 + (lx1 - lx0) * ty;
                }
            });
        out.into_vec()
    }
}
