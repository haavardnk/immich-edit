use rayon::prelude::*;

use super::estimate::NoiseCurve;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelParams {
    pub lambda: f32,
    pub mu: f32,
    pub keep: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Level<const C: usize> {
    pub step: usize,
    pub curves: [NoiseCurve; C],
    pub params: LevelParams,
}

pub(crate) fn amount_removed(amount: f32) -> f32 {
    (amount / 100.0).clamp(0.0, 1.0)
}

pub(crate) fn detail_mu(detail: f32) -> f32 {
    ((50.0 - detail) / 50.0).exp2()
}

#[inline(always)]
pub(crate) fn gain(energy: f32, variance: f32, p: LevelParams) -> f32 {
    let noise = variance * p.lambda;
    if noise <= 0.0 {
        return 1.0;
    }
    let signal = (energy - p.mu * noise).max(0.0);
    let g = signal / (signal + noise);
    g + (1.0 - g) * p.keep
}

fn energy_row(rows: [&[f32]; 3], step: usize, vsum: &mut [f32], out: &mut [f32]) {
    let w = out.len();
    for (x, v) in vsum.iter_mut().enumerate() {
        *v = rows[0][x] * rows[0][x] + rows[1][x] * rows[1][x] + rows[2][x] * rows[2][x];
    }
    for (x, e) in out.iter_mut().enumerate() {
        let l = x.saturating_sub(step);
        let r = (x + step).min(w - 1);
        let centre = rows[1][x] * rows[1][x];
        let around = vsum[l] + vsum[x] + vsum[r] - centre;
        *e = (around + centre.min(around / 8.0)) / 9.0;
    }
}

struct RowScratch<const C: usize> {
    rows: [[Vec<f32>; 3]; C],
    energy: [Vec<f32>; C],
    delta: [Vec<f32>; C],
    vsum: Vec<f32>,
    reference: Vec<f32>,
}

impl<const C: usize> RowScratch<C> {
    fn new(w: usize) -> Self {
        Self {
            rows: std::array::from_fn(|_| std::array::from_fn(|_| vec![0.0; w])),
            energy: std::array::from_fn(|_| vec![0.0; w]),
            delta: std::array::from_fn(|_| vec![0.0; w]),
            vsum: vec![0.0; w],
            reference: vec![0.0; w],
        }
    }
}

pub(crate) fn shrink_rows<const C: usize, D, R, W>(
    out: &mut [f32],
    w: usize,
    h: usize,
    level: Level<C>,
    detail: D,
    reference: R,
    write: W,
) where
    D: Fn(usize, usize, &mut [f32]) + Sync,
    R: Fn(usize, &mut [f32]) + Sync,
    W: Fn(usize, &mut [f32], [&[f32]; C]) + Sync,
{
    let stride = out.len() / h;
    let step = level.step as isize;
    out.par_chunks_mut(stride).enumerate().for_each_init(
        || RowScratch::<C>::new(w),
        |s, (y, row)| {
            let neighbours =
                [-step, 0, step].map(|dy| (y as isize + dy).clamp(0, h as isize - 1) as usize);
            for (c, rows) in s.rows.iter_mut().enumerate() {
                for (buf, &yy) in rows.iter_mut().zip(&neighbours) {
                    detail(c, yy, buf);
                }
            }
            reference(y, &mut s.reference);
            for c in 0..C {
                let [up, mid, down] = &s.rows[c];
                energy_row([up, mid, down], level.step, &mut s.vsum, &mut s.energy[c]);
                for (((d, &e), &r), &m) in s.delta[c]
                    .iter_mut()
                    .zip(&s.energy[c])
                    .zip(&s.reference)
                    .zip(mid.iter())
                {
                    let g = gain(e, level.curves[c].variance(r), level.params);
                    *d = (g - 1.0) * m;
                }
            }
            write(y, row, std::array::from_fn(|c| s.delta[c].as_slice()));
        },
    );
}
