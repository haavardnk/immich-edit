use super::shrink::{self, Level, LevelParams};
use super::{LUMA_LEVELS, atrous, estimate};
use crate::cpu::scratch::Scratch;
use crate::math::luma;
use crate::ops::LinearImage;
use rayon::prelude::*;

pub(crate) fn level_params(amount: f32, detail: f32, contrast: f32) -> [LevelParams; LUMA_LEVELS] {
    let removed = shrink::amount_removed(amount);
    let mu = shrink::detail_mu(detail);
    std::array::from_fn(|s| LevelParams {
        lambda: 1.0,
        mu,
        keep: 1.0 - removed * (1.0 - contrast / 100.0 * s as f32 / (LUMA_LEVELS - 1) as f32),
    })
}

pub(crate) fn denoise(image: &mut LinearImage, params: [LevelParams; LUMA_LEVELS]) {
    let w = image.width;
    let h = image.height;
    if w < 3 || h < 3 {
        return;
    }
    let n = w * h;
    let mut cur = Scratch::zeroed(n);
    cur.par_chunks_mut(w)
        .zip(image.rgb.par_chunks(w * 3))
        .for_each(|(out, rgb)| {
            for (o, px) in out.iter_mut().zip(rgb.chunks_exact(3)) {
                *o = luma(px[0], px[1], px[2]);
            }
        });
    let mut next = Scratch::zeroed(n);
    let mut tmp = Scratch::zeroed(n);
    let mut acc = Scratch::zeroed(n);
    let mut coarser_than = None;
    for (s, p) in params.into_iter().enumerate() {
        let step = 1 << s;
        atrous::smooth(&cur, &mut next, &mut tmp, w, h, step);
        let (c, nx): (&[f32], &[f32]) = (&cur, &next);
        let hist = estimate::histogram(w, h, |x, y| {
            let i = y * w + x;
            (c[i] - nx[i], nx[i])
        });
        let curve = estimate::fit(&hist, coarser_than);
        coarser_than = Some(curve);
        shrink::shrink_rows(
            &mut acc,
            w,
            h,
            Level {
                step,
                curves: [curve],
                params: p,
            },
            |_, y, buf| {
                let row = y * w..(y + 1) * w;
                for ((o, a), b) in buf.iter_mut().zip(&c[row.clone()]).zip(&nx[row]) {
                    *o = a - b;
                }
            },
            |y, buf| buf.copy_from_slice(&nx[y * w..(y + 1) * w]),
            |_, row, [delta]| {
                for (o, d) in row.iter_mut().zip(delta) {
                    *o += d;
                }
            },
        );
        std::mem::swap(&mut cur, &mut next);
    }
    image
        .rgb
        .par_chunks_mut(w * 3)
        .zip(acc.par_chunks(w))
        .for_each(|(rgb, deltas)| {
            for (px, d) in rgb.chunks_exact_mut(3).zip(deltas) {
                let y0 = luma(px[0], px[1], px[2]);
                let y1 = y0 + d;
                let r = chroma_scale(y0, y1);
                px[0] = y1 + (px[0] - y0) * r;
                px[1] = y1 + (px[1] - y0) * r;
                px[2] = y1 + (px[2] - y0) * r;
            }
        });
}

#[inline(always)]
fn chroma_scale(y0: f32, y1: f32) -> f32 {
    if y0 > 1e-6 && y1 > 0.0 {
        (y1 / y0).sqrt().clamp(0.5, 2.0)
    } else {
        1.0
    }
}
