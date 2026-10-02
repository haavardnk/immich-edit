use rayon::prelude::*;

use super::CaptureSigma;

const BAND: usize = 64;

pub fn gaussian_kernel(sigma: f32) -> Vec<f32> {
    let radius = (3.0 * sigma).ceil().max(1.0) as usize;
    let mut kernel: Vec<f32> = (0..=2 * radius)
        .map(|i| {
            let d = i as f32 - radius as f32;
            (-(d * d) / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let sum: f32 = kernel.iter().sum();
    for k in kernel.iter_mut() {
        *k /= sum;
    }
    kernel
}

pub fn half_diagonal(w: usize, h: usize) -> f32 {
    0.5 * (w as f32).hypot(h as f32)
}

pub fn pixel_level(x: usize, y: usize, w: usize, h: usize, half_diag: f32, levels: usize) -> u8 {
    let dx = x as f32 + 0.5 - 0.5 * w as f32;
    let dy = y as f32 + 0.5 - 0.5 * h as f32;
    let t = (dx * dx + dy * dy).sqrt() / half_diag;
    ((t * (levels - 1) as f32 + 0.5).floor() as usize).min(levels - 1) as u8
}

pub struct Psf {
    kernels: Vec<Vec<f32>>,
    levels: Vec<u8>,
    width: usize,
}

impl Psf {
    pub fn new(sigma: CaptureSigma, w: usize, h: usize) -> Self {
        let kernels = sigma.kernels();
        let count = kernels.len();
        let mut levels = vec![0u8; w * h];
        if count > 1 {
            let half_diag = half_diagonal(w, h);
            levels.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
                for (x, level) in row.iter_mut().enumerate() {
                    *level = pixel_level(x, y, w, h, half_diag, count);
                }
            });
        }
        Self {
            kernels,
            levels,
            width: w,
        }
    }

    fn reach(&self) -> usize {
        self.kernels.iter().map(|k| k.len() / 2).max().unwrap_or(0)
    }

    fn runs(&self, y: usize) -> impl Iterator<Item = (usize, usize, &[f32])> {
        self.levels[y * self.width..(y + 1) * self.width]
            .chunk_by(|a, b| a == b)
            .scan(0, |start, run| {
                let span = (
                    *start,
                    *start + run.len(),
                    self.kernels[usize::from(run[0])].as_slice(),
                );
                *start += run.len();
                Some(span)
            })
    }
}

fn blur_span(src_row: &[f32], out: &mut [f32], x0: usize, kernel: &[f32]) {
    let w = src_row.len();
    let radius = kernel.len() / 2;
    let x1 = x0 + out.len();
    let lo = radius.clamp(x0, x1);
    let hi = w.saturating_sub(radius).clamp(lo, x1);
    for x in (x0..lo).chain(hi..x1) {
        let mut acc = 0.0f32;
        for (i, k) in kernel.iter().enumerate() {
            let sx = (x + i).saturating_sub(radius).min(w - 1);
            acc += k * src_row[sx];
        }
        out[x - x0] = acc;
    }
    if hi <= lo {
        return;
    }
    let inner = &mut out[lo - x0..hi - x0];
    inner.fill(0.0);
    for (i, k) in kernel.iter().enumerate() {
        let taps = &src_row[lo + i - radius..lo + i - radius + inner.len()];
        for (o, v) in inner.iter_mut().zip(taps) {
            *o += k * v;
        }
    }
}

pub fn convolve_with<F>(src: &[f32], dst: &mut [f32], w: usize, h: usize, psf: &Psf, finish: F)
where
    F: Fn(usize, &[f32], &mut [f32]) + Sync,
{
    let reach = psf.reach();
    dst.par_chunks_mut(w * BAND).enumerate().for_each_init(
        || (Vec::new(), vec![0.0f32; w]),
        |(tmp, acc), (band, out)| {
            let y0 = band * BAND;
            let top = y0.saturating_sub(reach);
            let bottom = (y0 + out.len() / w + reach).min(h);
            tmp.resize((bottom - top) * w, 0.0);
            for (sy, row) in (top..bottom).zip(tmp.chunks_exact_mut(w)) {
                let src_row = &src[sy * w..sy * w + w];
                for (start, end, kernel) in psf.runs(sy) {
                    blur_span(src_row, &mut row[start..end], start, kernel);
                }
            }
            for (y, row) in (y0..).zip(out.chunks_exact_mut(w)) {
                acc.fill(0.0);
                for (start, end, kernel) in psf.runs(y) {
                    let radius = kernel.len() / 2;
                    let span = &mut acc[start..end];
                    for (i, k) in kernel.iter().enumerate() {
                        let sy = (y + i).saturating_sub(radius).min(h - 1) - top;
                        for (a, v) in span.iter_mut().zip(&tmp[sy * w + start..sy * w + end]) {
                            *a += k * v;
                        }
                    }
                }
                finish(y, acc, row);
            }
        },
    );
}

fn extreme_row<P: Fn(f32, f32) -> f32>(src_row: &[f32], row: &mut [f32], radius: usize, pick: P) {
    let w = src_row.len();
    row.copy_from_slice(src_row);
    let left_end = radius.min(w);
    let right_start = w.saturating_sub(radius).max(left_end);
    for x in (0..left_end).chain(right_start..w) {
        for d in 1..=radius {
            let lo = x.saturating_sub(d);
            let hi = (x + d).min(w - 1);
            row[x] = pick(pick(row[x], src_row[lo]), src_row[hi]);
        }
    }
    let n = right_start - left_end;
    for d in 1..=radius.min(left_end) {
        let before = &src_row[left_end - d..left_end - d + n];
        let after = &src_row[left_end + d..left_end + d + n];
        for ((out, a), b) in row[left_end..right_start].iter_mut().zip(before).zip(after) {
            *out = pick(pick(*out, *a), *b);
        }
    }
}

pub fn local_extreme<P: Fn(f32, f32) -> f32 + Sync + Copy>(
    src: &[f32],
    dst: &mut [f32],
    w: usize,
    h: usize,
    psf: &Psf,
    pick: P,
) {
    let reach = psf.reach();
    dst.par_chunks_mut(w * BAND).enumerate().for_each_init(
        || (vec![Vec::new(); reach + 1], vec![false; reach + 1]),
        |(rows, present), (band, out)| {
            let y0 = band * BAND;
            let y1 = y0 + out.len() / w;
            let top = y0.saturating_sub(reach);
            let bottom = (y1 + reach).min(h);
            present.fill(false);
            for y in y0..y1 {
                for (_, _, kernel) in psf.runs(y) {
                    present[kernel.len() / 2] = true;
                }
            }
            for (radius, buf) in rows.iter_mut().enumerate().filter(|(r, _)| present[*r]) {
                buf.resize((bottom - top) * w, 0.0);
                for (sy, row) in (top..bottom).zip(buf.chunks_exact_mut(w)) {
                    extreme_row(&src[sy * w..sy * w + w], row, radius, pick);
                }
            }
            for (y, row) in (y0..).zip(out.chunks_exact_mut(w)) {
                for (start, end, kernel) in psf.runs(y) {
                    let radius = kernel.len() / 2;
                    let buf = &rows[radius];
                    let lo = y.saturating_sub(radius) - top;
                    let hi = (y + radius).min(h - 1) - top;
                    let span = &mut row[start..end];
                    span.copy_from_slice(&buf[lo * w + start..lo * w + end]);
                    for sy in lo + 1..=hi {
                        for (o, v) in span.iter_mut().zip(&buf[sy * w + start..sy * w + end]) {
                            *o = pick(*o, *v);
                        }
                    }
                }
            }
        },
    );
}
