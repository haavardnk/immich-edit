use crate::cpu::scratch::Scratch;
use rayon::prelude::*;

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

pub(crate) fn gaussian_blur_rgb(src: &[f32], w: usize, h: usize, kernel: &[f32]) -> Scratch {
    let radius = kernel.len() / 2;
    let mut tmp = Scratch::take_uninit(src.len());
    tmp.par_chunks_mut(w * 3)
        .zip(src.par_chunks(w * 3))
        .for_each(|(dst_row, src_row)| {
            for x in 0..w {
                let mut acc = [0.0f32; 3];
                for (k, weight) in kernel.iter().enumerate() {
                    let sx = (x as isize + k as isize - radius as isize).clamp(0, w as isize - 1)
                        as usize;
                    let si = sx * 3;
                    acc[0] += src_row[si] * weight;
                    acc[1] += src_row[si + 1] * weight;
                    acc[2] += src_row[si + 2] * weight;
                }
                let di = x * 3;
                dst_row[di] = acc[0];
                dst_row[di + 1] = acc[1];
                dst_row[di + 2] = acc[2];
            }
        });
    let mut out = Scratch::take_uninit(src.len());
    out.par_chunks_mut(w * 3)
        .enumerate()
        .for_each(|(y, dst_row)| {
            for x in 0..w {
                let mut acc = [0.0f32; 3];
                for (k, weight) in kernel.iter().enumerate() {
                    let sy = (y as isize + k as isize - radius as isize).clamp(0, h as isize - 1)
                        as usize;
                    let si = (sy * w + x) * 3;
                    acc[0] += tmp[si] * weight;
                    acc[1] += tmp[si + 1] * weight;
                    acc[2] += tmp[si + 2] * weight;
                }
                let di = x * 3;
                dst_row[di] = acc[0];
                dst_row[di + 1] = acc[1];
                dst_row[di + 2] = acc[2];
            }
        });
    out
}
