use crate::cpu::scratch::Scratch;
use rayon::prelude::*;

#[cfg(feature = "native")]
pub(crate) fn gaussian_kernel(sigma: f32) -> Vec<f32> {
    let s = sigma.max(0.01);
    let radius = (s * 3.0).ceil() as usize;
    let size = radius * 2 + 1;
    let mut k = vec![0.0f32; size];
    let two_s2 = 2.0 * s * s;
    let mut sum = 0.0;
    for (i, slot) in k.iter_mut().enumerate() {
        let x = i as f32 - radius as f32;
        let v = (-(x * x) / two_s2).exp();
        *slot = v;
        sum += v;
    }
    for slot in &mut k {
        *slot /= sum;
    }
    k
}

pub(crate) fn discrete_gaussian_kernel(sigma: f32) -> Vec<f32> {
    let s = sigma.max(0.01);
    let radius = (s * 3.0).ceil() as usize;
    let t = f64::from(s * s);
    let half: Vec<f64> = (0..=radius).map(|n| scaled_bessel_i(n, t)).collect();
    let sum = half[0] + 2.0 * half[1..].iter().sum::<f64>();
    (0..=2 * radius)
        .map(|i| (half[i.abs_diff(radius)] / sum) as f32)
        .collect()
}

fn scaled_bessel_i(n: usize, t: f64) -> f64 {
    let x = t / 2.0;
    let factorial: f64 = (1..=n).map(|k| k as f64).product();
    let mut term = (-t).exp() * x.powi(n as i32) / factorial;
    let mut sum = term;
    for k in 1..200 {
        term *= x * x / (k * (k + n)) as f64;
        sum += term;
        if term <= sum * 1e-12 {
            break;
        }
    }
    sum
}

pub(crate) fn gaussian_blur<const C: usize>(
    src: &[f32],
    w: usize,
    h: usize,
    kernel: &[f32],
) -> Scratch {
    let radius = kernel.len() / 2;
    let mut tmp = Scratch::zeroed(src.len());
    tmp.par_chunks_mut(w * C)
        .zip(src.par_chunks(w * C))
        .for_each(|(dst_row, src_row)| {
            for x in 0..w {
                let mut acc = [0.0f32; C];
                for (k, weight) in kernel.iter().enumerate() {
                    let sx = (x as isize + k as isize - radius as isize).clamp(0, w as isize - 1)
                        as usize;
                    let si = sx * C;
                    for (c, a) in acc.iter_mut().enumerate() {
                        *a += src_row[si + c] * weight;
                    }
                }
                dst_row[x * C..x * C + C].copy_from_slice(&acc);
            }
        });
    let mut out = Scratch::zeroed(src.len());
    out.par_chunks_mut(w * C)
        .enumerate()
        .for_each(|(y, dst_row)| {
            for x in 0..w {
                let mut acc = [0.0f32; C];
                for (k, weight) in kernel.iter().enumerate() {
                    let sy = (y as isize + k as isize - radius as isize).clamp(0, h as isize - 1)
                        as usize;
                    let si = (sy * w + x) * C;
                    for (c, a) in acc.iter_mut().enumerate() {
                        *a += tmp[si + c] * weight;
                    }
                }
                dst_row[x * C..x * C + C].copy_from_slice(&acc);
            }
        });
    out
}
