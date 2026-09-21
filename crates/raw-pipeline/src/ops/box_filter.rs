use crate::cpu::scratch::Scratch;
use rayon::prelude::*;
use std::collections::VecDeque;

const TRANSPOSE_BLOCK: usize = 32;

type RowPass = fn(&[f32], &mut [f32], usize, usize);

pub(crate) fn transpose(src: &[f32], dst: &mut [f32], w: usize, h: usize) {
    if w == 0 || h == 0 {
        return;
    }
    dst.par_chunks_mut(h * TRANSPOSE_BLOCK)
        .enumerate()
        .for_each(|(block, cols)| {
            let x0 = block * TRANSPOSE_BLOCK;
            for y0 in (0..h).step_by(TRANSPOSE_BLOCK) {
                let y1 = (y0 + TRANSPOSE_BLOCK).min(h);
                for (dx, col) in cols.chunks_exact_mut(h).enumerate() {
                    let x = x0 + dx;
                    for (i, v) in col[y0..y1].iter_mut().enumerate() {
                        *v = src[(y0 + i) * w + x];
                    }
                }
            }
        });
}

fn box_mean_rows(src: &[f32], dst: &mut [f32], w: usize, r: usize) {
    dst.par_chunks_exact_mut(w)
        .zip(src.par_chunks_exact(w))
        .for_each(|(d, s)| {
            let mut sum: f32 = 0.0;
            for v in s.iter().take(r.min(w)) {
                sum += *v;
            }
            for (x, dv) in d.iter_mut().enumerate() {
                let add = x + r;
                if add < w {
                    sum += s[add];
                }
                let rem = x as isize - r as isize - 1;
                if rem >= 0 {
                    sum -= s[rem as usize];
                }
                let lo = rem.max(-1) + 1;
                let hi = (add.min(w - 1)) as isize;
                let count = (hi - lo + 1) as f32;
                *dv = sum / count;
            }
        });
}

fn min_filter_rows(src: &[f32], dst: &mut [f32], w: usize, r: usize) {
    dst.par_chunks_exact_mut(w)
        .zip(src.par_chunks_exact(w))
        .for_each(|(d, s)| {
            let mut deque: VecDeque<usize> = VecDeque::new();
            for x in 0..w + r {
                if x < w {
                    while let Some(&back) = deque.back() {
                        if s[back] >= s[x] {
                            deque.pop_back();
                        } else {
                            break;
                        }
                    }
                    deque.push_back(x);
                }
                let lo = x as isize - 2 * r as isize;
                while let Some(&front) = deque.front() {
                    if (front as isize) < lo {
                        deque.pop_front();
                    } else {
                        break;
                    }
                }
                if x >= r {
                    let Some(&front) = deque.front() else {
                        continue;
                    };
                    d[x - r] = s[front];
                }
            }
        });
}

fn separable(src: &[f32], w: usize, h: usize, r: usize, rows: RowPass) -> Scratch {
    let n = w * h;
    let mut out = Scratch::zeroed(n);
    if n == 0 {
        return out;
    }
    let mut pass = Scratch::zeroed(n);
    let mut flipped = Scratch::zeroed(n);
    rows(src, &mut pass, w, r);
    transpose(&pass, &mut flipped, w, h);
    rows(&flipped, &mut pass, h, r);
    transpose(&pass, &mut out, h, w);
    out
}

pub(crate) fn box_mean(src: &[f32], w: usize, h: usize, r: usize) -> Scratch {
    separable(src, w, h, r, box_mean_rows)
}

pub(crate) fn min_filter(src: &[f32], w: usize, h: usize, r: usize) -> Scratch {
    separable(src, w, h, r, min_filter_rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(w: usize, h: usize) -> Vec<f32> {
        (0..w * h)
            .map(|i| ((i * 37 % 101) as f32) / 101.0)
            .collect()
    }

    fn brute_box_mean(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
        let window = |c: usize, len: usize| {
            let lo = c.saturating_sub(r);
            let hi = (c + r).min(len - 1);
            lo..=hi
        };
        let mut horizontal = vec![0.0f32; w * h];
        for (i, v) in horizontal.iter_mut().enumerate() {
            let x = i % w;
            let y = i / w;
            let span = window(x, w);
            let count = span.clone().count() as f32;
            *v = span.map(|sx| src[y * w + sx]).sum::<f32>() / count;
        }
        let mut out = vec![0.0f32; w * h];
        for (i, v) in out.iter_mut().enumerate() {
            let x = i % w;
            let y = i / w;
            let span = window(y, h);
            let count = span.clone().count() as f32;
            *v = span.map(|sy| horizontal[sy * w + x]).sum::<f32>() / count;
        }
        out
    }

    fn brute_min_filter(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
        let mut out = vec![0.0f32; w * h];
        for (i, v) in out.iter_mut().enumerate() {
            let x = i % w;
            let y = i / w;
            let mut best = f32::INFINITY;
            let ys = y.saturating_sub(r)..=(y + r).min(h - 1);
            for sy in ys {
                let xs = x.saturating_sub(r)..=(x + r).min(w - 1);
                for sx in xs {
                    best = best.min(src[sy * w + sx]);
                }
            }
            *v = best;
        }
        out
    }

    #[test]
    fn transpose_round_trips() {
        let (w, h) = (71, 43);
        let src = ramp(w, h);
        let mut flipped = vec![0.0f32; w * h];
        transpose(&src, &mut flipped, w, h);
        let mut back = vec![0.0f32; w * h];
        transpose(&flipped, &mut back, h, w);
        if back != src {
            panic!("transpose is not an involution");
        }
    }

    #[test]
    fn box_mean_matches_brute_force() {
        let (w, h, r) = (37, 29, 4);
        let src = ramp(w, h);
        let got = box_mean(&src, w, h, r);
        let want = brute_box_mean(&src, w, h, r);
        for (i, (a, b)) in got.iter().zip(want.iter()).enumerate() {
            if (a - b).abs() > 1e-5 {
                panic!("box_mean differs at {i}: {a} vs {b}");
            }
        }
    }

    #[test]
    fn min_filter_matches_brute_force() {
        let (w, h, r) = (37, 29, 3);
        let src = ramp(w, h);
        let got = min_filter(&src, w, h, r);
        let want = brute_min_filter(&src, w, h, r);
        for (i, (a, b)) in got.iter().zip(want.iter()).enumerate() {
            if (a - b).abs() > 1e-6 {
                panic!("min_filter differs at {i}: {a} vs {b}");
            }
        }
    }

    #[test]
    fn degenerate_dimensions_do_not_panic() {
        for (w, h) in [(1usize, 1usize), (1, 9), (9, 1), (0, 0), (0, 5), (5, 0)] {
            let src = vec![0.5f32; w * h];
            let mean = box_mean(&src, w, h, 4);
            let min = min_filter(&src, w, h, 4);
            if mean.len() != w * h || min.len() != w * h {
                panic!("wrong length for {w}x{h}");
            }
            if w * h > 0 && (mean[0] - 0.5).abs() > 1e-6 {
                panic!("constant input should survive the box mean at {w}x{h}");
            }
        }
    }
}
