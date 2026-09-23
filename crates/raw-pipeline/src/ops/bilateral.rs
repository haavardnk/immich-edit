use multiversion::multiversion;
use rayon::prelude::*;

use crate::vmath;

#[derive(Clone, Copy)]
pub(crate) struct Bilateral {
    pub radius: usize,
    pub inv_2ss: f32,
    pub inv_2sr: f32,
}

struct Sums<const C: usize> {
    wsum: Vec<f32>,
    acc: [Vec<f32>; C],
}

impl<const C: usize> Sums<C> {
    fn new(width: usize) -> Self {
        Self {
            wsum: vec![0.0; width],
            acc: std::array::from_fn(|_| vec![0.0; width]),
        }
    }
}

pub(crate) fn filter<const C: usize>(
    src: [&[f32]; C],
    dst: [&mut [f32]; C],
    w: usize,
    h: usize,
    p: Bilateral,
) {
    let rows: Vec<[&mut [f32]; C]> = {
        let mut chunks = dst.map(|d| d.chunks_mut(w));
        (0..h)
            .map(|_| std::array::from_fn(|c| chunks[c].next().unwrap_or_default()))
            .collect()
    };
    rows.into_par_iter().enumerate().for_each_init(
        || Sums::<C>::new(w),
        |sums, (y, out)| filter_row(src, out, w, h, y, p, sums),
    );
}

#[multiversion(targets("x86_64+avx2", "x86_64+sse4.2"))]
fn filter_row<const C: usize>(
    src: [&[f32]; C],
    out: [&mut [f32]; C],
    w: usize,
    h: usize,
    y: usize,
    p: Bilateral,
    sums: &mut Sums<C>,
) {
    let r = p.radius;
    let y0 = y.saturating_sub(r);
    let y1 = (y + r).min(h - 1);
    let lo = r.min(w);
    let hi = w.saturating_sub(r).max(lo);
    let center: [&[f32]; C] = std::array::from_fn(|c| &src[c][y * w..(y + 1) * w]);
    let len = hi - lo;
    let Sums { wsum, acc } = sums;
    let wsum = &mut wsum[..len];
    let mut acc: [&mut [f32]; C] = acc.each_mut().map(|a| &mut a[..len]);
    wsum.fill(0.0);
    for a in &mut acc {
        a.fill(0.0);
    }
    let mid: [&[f32]; C] = std::array::from_fn(|c| &center[c][lo..hi]);
    let taps = (y0..=y1)
        .flat_map(|yy| (0..=2 * r).map(move |dxi| (yy, dxi)))
        .filter(|_| len > 0);
    for (yy, dxi) in taps {
        let dy = yy as f32 - y as f32;
        let dx = dxi as f32 - r as f32;
        let spatial = -(dx * dx + dy * dy) * p.inv_2ss;
        let shift = lo + dxi - r;
        let tap: [&[f32]; C] = std::array::from_fn(|c| &src[c][yy * w + shift..][..len]);
        for i in 0..len {
            let mut dr2 = 0.0f32;
            for c in 0..C {
                let d = tap[c][i] - mid[c][i];
                dr2 += d * d;
            }
            let wgt = vmath::exp(spatial - dr2 * p.inv_2sr);
            wsum[i] += wgt;
            for c in 0..C {
                acc[c][i] += wgt * tap[c][i];
            }
        }
    }
    for i in 0..len {
        for c in 0..C {
            out[c][lo + i] = if wsum[i] > 0.0 {
                acc[c][i] / wsum[i]
            } else {
                mid[c][i]
            };
        }
    }
    for x in (0..lo).chain(hi..w) {
        let values = filter_edge_pixel(src, w, x, y, y0, y1, p);
        for c in 0..C {
            out[c][x] = values[c];
        }
    }
}

#[inline(always)]
fn filter_edge_pixel<const C: usize>(
    src: [&[f32]; C],
    w: usize,
    x: usize,
    y: usize,
    y0: usize,
    y1: usize,
    p: Bilateral,
) -> [f32; C] {
    let center: [f32; C] = std::array::from_fn(|c| src[c][y * w + x]);
    let x0 = x.saturating_sub(p.radius);
    let x1 = (x + p.radius).min(w - 1);
    let mut wsum = 0.0f32;
    let mut acc = [0.0f32; C];
    for (yy, xx) in (y0..=y1).flat_map(|yy| (x0..=x1).map(move |xx| (yy, xx))) {
        let dx = xx as f32 - x as f32;
        let dy = yy as f32 - y as f32;
        let mut dr2 = 0.0f32;
        for c in 0..C {
            let d = src[c][yy * w + xx] - center[c];
            dr2 += d * d;
        }
        let wgt = vmath::exp(-(dx * dx + dy * dy) * p.inv_2ss - dr2 * p.inv_2sr);
        wsum += wgt;
        for c in 0..C {
            acc[c] += wgt * src[c][yy * w + xx];
        }
    }
    std::array::from_fn(|c| if wsum > 0.0 { acc[c] / wsum } else { center[c] })
}

#[cfg(test)]
mod tests;
