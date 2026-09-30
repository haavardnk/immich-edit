use rayon::prelude::*;

pub(crate) const B3: [f32; 3] = [0.375, 0.25, 0.0625];

pub(crate) fn smooth(
    src: &[f32],
    dst: &mut [f32],
    tmp: &mut [f32],
    w: usize,
    h: usize,
    step: usize,
) {
    tmp.par_chunks_mut(w)
        .zip(src.par_chunks(w))
        .for_each(|(out, row)| smooth_row(row, out, step));
    let tmp: &[f32] = tmp;
    dst.par_chunks_mut(w).enumerate().for_each(|(y, out)| {
        let row = |dy: isize| {
            let yy = (y as isize + dy).clamp(0, h as isize - 1) as usize;
            &tmp[yy * w..(yy + 1) * w]
        };
        let s = step as isize;
        let (m2, m1, c, p1, p2) = (row(-2 * s), row(-s), row(0), row(s), row(2 * s));
        for (i, o) in out.iter_mut().enumerate() {
            *o = B3[0] * c[i] + B3[1] * (m1[i] + p1[i]) + B3[2] * (m2[i] + p2[i]);
        }
    });
}

fn smooth_row(row: &[f32], out: &mut [f32], step: usize) {
    let w = row.len();
    let at = |x: isize| row[x.clamp(0, w as isize - 1) as usize];
    let edge = (2 * step).min(w);
    let hi = w.saturating_sub(2 * step).max(edge);
    let s = step as isize;
    for x in (0..edge).chain(hi..w) {
        let xi = x as isize;
        out[x] = B3[0] * at(xi)
            + B3[1] * (at(xi - s) + at(xi + s))
            + B3[2] * (at(xi - 2 * s) + at(xi + 2 * s));
    }
    let n = hi - edge;
    if n == 0 {
        return;
    }
    let c = &row[edge..hi];
    let m1 = &row[edge - step..][..n];
    let p1 = &row[edge + step..][..n];
    let m2 = &row[edge - 2 * step..][..n];
    let p2 = &row[edge + 2 * step..][..n];
    for (i, o) in out[edge..hi].iter_mut().enumerate() {
        *o = B3[0] * c[i] + B3[1] * (m1[i] + p1[i]) + B3[2] * (m2[i] + p2[i]);
    }
}
