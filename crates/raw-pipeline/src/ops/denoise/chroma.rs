use super::shrink::{self, Level, LevelParams};
use super::{CHROMA_LEVELS, PB_DEN, PR_DEN, atrous, estimate};
use crate::cpu::scratch::Scratch;
use crate::math::luma;
use crate::ops::LinearImage;
use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};
use rayon::prelude::*;

pub(crate) fn level_params(
    amount: f32,
    detail: f32,
    smoothness: f32,
) -> [LevelParams; CHROMA_LEVELS + 1] {
    let removed = shrink::amount_removed(amount);
    let mu = shrink::detail_mu(detail);
    std::array::from_fn(|s| LevelParams {
        lambda: 1.0 + smoothness / 100.0 * s as f32 / 2.0,
        mu,
        keep: (1.0 - removed).powi(2),
    })
}

#[inline(always)]
pub(crate) fn to_chroma(r: f32, g: f32, b: f32) -> [f32; 3] {
    let y = luma(r, g, b);
    [y, (b - y) / PB_DEN, (r - y) / PR_DEN]
}

#[inline(always)]
pub(crate) fn from_chroma(y: f32, pb: f32, pr: f32) -> [f32; 3] {
    let r = y + PR_DEN * pr;
    let b = y + PB_DEN * pb;
    let g = (y - LUMA_R * r - LUMA_B * b) / LUMA_G;
    [r, g, b]
}

#[inline(always)]
fn taps(x: usize, n: usize) -> ([usize; 2], [f32; 2]) {
    let half = x / 2;
    if x % 2 == 0 {
        ([half.saturating_sub(1), half.min(n - 1)], [0.25, 0.75])
    } else {
        ([half.min(n - 1), (half + 1).min(n - 1)], [0.75, 0.25])
    }
}

#[inline(always)]
fn up_at(
    plane: &[f32],
    lw: usize,
    lh: usize,
    channels: usize,
    c: usize,
    x: usize,
    y: usize,
) -> f32 {
    let ([y0, y1], [wy0, wy1]) = taps(y, lh);
    let ([x0, x1], [wx0, wx1]) = taps(x, lw);
    let at = |yy: usize, xx: usize| plane[(yy * lw + xx) * channels + c];
    let a = wx0 * at(y0, x0) + wx1 * at(y0, x1);
    let b = wx0 * at(y1, x0) + wx1 * at(y1, x1);
    wy0 * a + wy1 * b
}

fn downsample(image: &LinearImage, lw: usize, y_lo: &mut [f32], chroma_lo: [&mut [f32]; 2]) {
    let w = image.width;
    let h = image.height;
    let [pb_lo, pr_lo] = chroma_lo;
    (
        y_lo.par_chunks_mut(lw),
        pb_lo.par_chunks_mut(lw),
        pr_lo.par_chunks_mut(lw),
    )
        .into_par_iter()
        .enumerate()
        .for_each(|(j, (yrow, pbrow, prrow))| {
            let r0 = (2 * j).min(h - 1);
            let r1 = (2 * j + 1).min(h - 1);
            let px = |yy: usize, xx: usize, k: usize| image.rgb[(yy * w + xx) * 3 + k];
            for i in 0..lw {
                let x0 = (2 * i).min(w - 1);
                let x1 = (2 * i + 1).min(w - 1);
                let avg: [f32; 3] = std::array::from_fn(|k| {
                    ((px(r0, x0, k) + px(r0, x1, k)) + (px(r1, x0, k) + px(r1, x1, k))) * 0.25
                });
                let [yv, pb, pr] = to_chroma(avg[0], avg[1], avg[2]);
                yrow[i] = yv;
                pbrow[i] = pb;
                prrow[i] = pr;
            }
        });
}

pub(crate) fn denoise(image: &mut LinearImage, params: [LevelParams; CHROMA_LEVELS + 1]) {
    let w = image.width;
    let h = image.height;
    if w < 3 || h < 3 {
        return;
    }
    let lw = w.div_ceil(2);
    let lh = h.div_ceil(2);
    let ln = lw * lh;
    let mut y_cur = Scratch::zeroed(ln);
    let mut base = [Scratch::zeroed(ln), Scratch::zeroed(ln)];
    {
        let [pb, pr] = &mut base;
        downsample(image, lw, &mut y_cur, [pb, pr]);
    }
    let mut y_next = Scratch::zeroed(ln);
    let mut cur = [Scratch::zeroed(ln), Scratch::zeroed(ln)];
    let mut next = [Scratch::zeroed(ln), Scratch::zeroed(ln)];
    let mut tmp = Scratch::zeroed(ln);
    let mut yref = Scratch::zeroed(ln);
    let mut den = Scratch::zeroed(2 * ln);
    for c in 0..2 {
        cur[c].copy_from_slice(&base[c]);
    }
    den.par_chunks_mut(2)
        .zip(base[0].par_iter().zip(base[1].par_iter()))
        .for_each(|(o, (pb, pr))| {
            o[0] = *pb;
            o[1] = *pr;
        });
    let mut coarser_than = [None; 2];
    for (s, p) in params[1..].iter().enumerate() {
        let step = 1 << s;
        atrous::smooth(&y_cur, &mut y_next, &mut tmp, lw, lh, step);
        for c in 0..2 {
            atrous::smooth(&cur[c], &mut next[c], &mut tmp, lw, lh, step);
        }
        if s == 0 {
            yref.copy_from_slice(&y_next);
        }
        let reference: &[f32] = &y_next;
        let (c_planes, n_planes) = (&cur, &next);
        let curves = std::array::from_fn(|c| {
            let hist = estimate::histogram(lw, lh, |x, y| {
                let i = y * lw + x;
                (c_planes[c][i] - n_planes[c][i], reference[i])
            });
            estimate::fit(&hist, coarser_than[c])
        });
        coarser_than = curves.map(Some);
        shrink::shrink_rows(
            &mut den,
            lw,
            lh,
            Level {
                step,
                curves,
                params: *p,
            },
            |c, y, buf| {
                let row = y * lw..(y + 1) * lw;
                for ((o, a), b) in buf
                    .iter_mut()
                    .zip(&c_planes[c][row.clone()])
                    .zip(&n_planes[c][row])
                {
                    *o = a - b;
                }
            },
            |y, buf| buf.copy_from_slice(&reference[y * lw..(y + 1) * lw]),
            |_, row, [dpb, dpr]| {
                for ((o, a), b) in row.chunks_exact_mut(2).zip(dpb).zip(dpr) {
                    o[0] += a;
                    o[1] += b;
                }
            },
        );
        std::mem::swap(&mut y_cur, &mut y_next);
        std::mem::swap(&mut cur, &mut next);
    }
    drop((y_cur, y_next, cur, next, tmp));

    let n = w * h;
    let mut fine = [Scratch::zeroed(n), Scratch::zeroed(n)];
    {
        let [fpb, fpr] = &mut fine;
        (
            fpb.par_chunks_mut(w),
            fpr.par_chunks_mut(w),
            image.rgb.par_chunks(w * 3),
        )
            .into_par_iter()
            .enumerate()
            .for_each(|(y, (fb, fr, rgb))| {
                for (x, px) in rgb.chunks_exact(3).enumerate() {
                    let [_, pb, pr] = to_chroma(px[0], px[1], px[2]);
                    fb[x] = pb - up_at(&base[0], lw, lh, 1, 0, x, y);
                    fr[x] = pr - up_at(&base[1], lw, lh, 1, 0, x, y);
                }
            });
    }
    let yref: &[f32] = &yref;
    let curves = std::array::from_fn(|c| {
        let plane: &[f32] = &fine[c];
        let hist = estimate::histogram(w, h, |x, y| {
            (plane[y * w + x], up_at(yref, lw, lh, 1, 0, x, y))
        });
        estimate::fit(&hist, None)
    });
    let den: &[f32] = &den;
    let fine_planes: [&[f32]; 2] = [&fine[0], &fine[1]];
    shrink::shrink_rows(
        &mut image.rgb,
        w,
        h,
        Level {
            step: 1,
            curves,
            params: params[0],
        },
        |c, y, buf| buf.copy_from_slice(&fine_planes[c][y * w..(y + 1) * w]),
        |y, buf| {
            for (x, o) in buf.iter_mut().enumerate() {
                *o = up_at(yref, lw, lh, 1, 0, x, y);
            }
        },
        |y, row, [dpb, dpr]| {
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let i = y * w + x;
                let yv = luma(px[0], px[1], px[2]);
                let pb = up_at(den, lw, lh, 2, 0, x, y) + fine_planes[0][i] + dpb[x];
                let pr = up_at(den, lw, lh, 2, 1, x, y) + fine_planes[1][i] + dpr[x];
                px.copy_from_slice(&from_chroma(yv, pb, pr));
            }
        },
    );
}
