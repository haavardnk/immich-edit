use crate::ops::LinearImage;
use rayon::prelude::*;

#[inline(always)]
fn luma(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn box_mean_h(src: &[f32], dst: &mut [f32], w: usize, _h: usize, r: usize) {
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

fn box_mean_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let src_addr = src.as_ptr() as usize;
    let dst_addr = dst.as_mut_ptr() as usize;
    (0..w).into_par_iter().for_each(|x| {
        let s_ptr = src_addr as *const f32;
        let d_ptr = dst_addr as *mut f32;
        let mut sum: f32 = 0.0;
        for y in 0..(r.min(h)) {
            unsafe {
                sum += *s_ptr.add(y * w + x);
            }
        }
        for y in 0..h {
            let add = y + r;
            if add < h {
                unsafe {
                    sum += *s_ptr.add(add * w + x);
                }
            }
            let rem = y as isize - r as isize - 1;
            if rem >= 0 {
                unsafe {
                    sum -= *s_ptr.add(rem as usize * w + x);
                }
            }
            let lo = rem.max(-1) + 1;
            let hi = (add.min(h - 1)) as isize;
            let count = (hi - lo + 1) as f32;
            unsafe {
                *d_ptr.add(y * w + x) = sum / count;
            }
        }
    });
}

fn box_mean(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut tmp = vec![0.0f32; w * h];
    let mut out = vec![0.0f32; w * h];
    box_mean_h(src, &mut tmp, w, h, r);
    box_mean_v(&tmp, &mut out, w, h, r);
    out
}

fn min_filter_h(src: &[f32], dst: &mut [f32], w: usize, _h: usize, r: usize) {
    dst.par_chunks_exact_mut(w)
        .zip(src.par_chunks_exact(w))
        .for_each(|(d, s)| {
            let mut deque: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
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
                let lo_isz = x as isize - 2 * r as isize;
                while let Some(&front) = deque.front() {
                    if (front as isize) < lo_isz {
                        deque.pop_front();
                    } else {
                        break;
                    }
                }
                if x >= r {
                    d[x - r] = s[*deque.front().unwrap()];
                }
            }
        });
}

fn min_filter_v(src: &[f32], dst: &mut [f32], w: usize, h: usize, r: usize) {
    let dst_addr = dst.as_mut_ptr() as usize;
    (0..w).into_par_iter().for_each(|x| {
        let dst_ptr = dst_addr as *mut f32;
        let mut deque: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        for y in 0..h + r {
            if y < h {
                let v = src[y * w + x];
                while let Some(&back) = deque.back() {
                    if src[back * w + x] >= v {
                        deque.pop_back();
                    } else {
                        break;
                    }
                }
                deque.push_back(y);
            }
            let lo_isz = y as isize - 2 * r as isize;
            while let Some(&front) = deque.front() {
                if (front as isize) < lo_isz {
                    deque.pop_front();
                } else {
                    break;
                }
            }
            if y >= r {
                let c = y - r;
                unsafe {
                    *dst_ptr.add(c * w + x) = src[*deque.front().unwrap() * w + x];
                }
            }
        }
    });
}

fn min_filter(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    let mut tmp = vec![0.0f32; w * h];
    let mut out = vec![0.0f32; w * h];
    min_filter_h(src, &mut tmp, w, h, r);
    min_filter_v(&tmp, &mut out, w, h, r);
    out
}

fn dark_channel_per_pixel(rgb: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; w * h];
    out.par_iter_mut().enumerate().for_each(|(i, v)| {
        let r = rgb[i * 3].clamp(0.0, 1.0);
        let g = rgb[i * 3 + 1].clamp(0.0, 1.0);
        let b = rgb[i * 3 + 2].clamp(0.0, 1.0);
        *v = r.min(g).min(b);
    });
    out
}

fn estimate_atmosphere(rgb: &[f32], dp: &[f32], w: usize, h: usize) -> [f32; 3] {
    let n = w * h;
    let take = (n / 1000).clamp(16, 256);
    let mut idx: Vec<u32> = (0..n as u32).collect();
    idx.select_nth_unstable_by(take, |&a, &b| {
        dp[b as usize]
            .partial_cmp(&dp[a as usize])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top = &idx[..take];
    let mut sr: f32 = 0.0;
    let mut sg: f32 = 0.0;
    let mut sb: f32 = 0.0;
    for &i in top {
        let j = i as usize * 3;
        sr += rgb[j];
        sg += rgb[j + 1];
        sb += rgb[j + 2];
    }
    let inv = 1.0 / take as f32;
    [
        (sr * inv).clamp(0.5, 1.0),
        (sg * inv).clamp(0.5, 1.0),
        (sb * inv).clamp(0.5, 1.0),
    ]
}

fn guided_filter(guide: &[f32], p: &[f32], w: usize, h: usize, r: usize, eps: f32) -> Vec<f32> {
    let mean_i = box_mean(guide, w, h, r);
    let mean_p = box_mean(p, w, h, r);
    let ii: Vec<f32> = guide.par_iter().map(|x| x * x).collect();
    let ip: Vec<f32> = guide
        .par_iter()
        .zip(p.par_iter())
        .map(|(a, b)| a * b)
        .collect();
    let corr_i = box_mean(&ii, w, h, r);
    let corr_ip = box_mean(&ip, w, h, r);
    let var_i: Vec<f32> = corr_i
        .par_iter()
        .zip(mean_i.par_iter())
        .map(|(c, m)| c - m * m)
        .collect();
    let cov_ip: Vec<f32> = corr_ip
        .par_iter()
        .zip(mean_i.par_iter())
        .zip(mean_p.par_iter())
        .map(|((c, mi), mp)| c - mi * mp)
        .collect();
    let a_coef: Vec<f32> = cov_ip
        .par_iter()
        .zip(var_i.par_iter())
        .map(|(c, v)| c / (v + eps))
        .collect();
    let b_coef: Vec<f32> = a_coef
        .par_iter()
        .zip(mean_i.par_iter())
        .zip(mean_p.par_iter())
        .map(|((a, mi), mp)| mp - a * mi)
        .collect();
    let mean_a = box_mean(&a_coef, w, h, r);
    let mean_b = box_mean(&b_coef, w, h, r);
    mean_a
        .par_iter()
        .zip(mean_b.par_iter())
        .zip(guide.par_iter())
        .map(|((ma, mb), g)| ma * g + mb)
        .collect()
}

pub fn apply_dehaze(image: &mut LinearImage, amount: f32) {
    if amount == 0.0 {
        return;
    }
    let a = amount.clamp(-1.0, 1.0);
    let w = image.width;
    let h = image.height;
    if w < 8 || h < 8 {
        return;
    }
    let min_dim = w.min(h);
    let r_patch = (min_dim / 200).max(8).min(min_dim / 2);
    let r_gf = (min_dim / 50).max(16).min(min_dim / 2);
    let d0 = dark_channel_per_pixel(&image.rgb, w, h);
    let dp = min_filter(&d0, w, h, r_patch);
    let atm = estimate_atmosphere(&image.rgb, &dp, w, h);
    let dn: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            let r = (image.rgb[i * 3] / atm[0]).clamp(0.0, 1.0);
            let g = (image.rgb[i * 3 + 1] / atm[1]).clamp(0.0, 1.0);
            let b = (image.rgb[i * 3 + 2] / atm[2]).clamp(0.0, 1.0);
            r.min(g).min(b)
        })
        .collect();
    let dn_patch = min_filter(&dn, w, h, r_patch);
    let t_raw: Vec<f32> = dn_patch
        .par_iter()
        .map(|d| (1.0 - 0.95 * d).clamp(0.0, 1.0))
        .collect();
    let guide: Vec<f32> = (0..w * h)
        .into_par_iter()
        .map(|i| {
            luma(
                image.rgb[i * 3].clamp(0.0, 1.0),
                image.rgb[i * 3 + 1].clamp(0.0, 1.0),
                image.rgb[i * 3 + 2].clamp(0.0, 1.0),
            )
        })
        .collect();
    let t = guided_filter(&guide, &t_raw, w, h, r_gf, 1e-3);
    if a > 0.0 {
        image
            .rgb
            .par_chunks_exact_mut(3)
            .enumerate()
            .for_each(|(i, px)| {
                let ti = t[i].max(0.16);
                let jr = (px[0] - atm[0]) / ti + atm[0];
                let jg = (px[1] - atm[1]) / ti + atm[1];
                let jb = (px[2] - atm[2]) / ti + atm[2];
                px[0] = px[0] + (jr.max(0.0) - px[0]) * a;
                px[1] = px[1] + (jg.max(0.0) - px[1]) * a;
                px[2] = px[2] + (jb.max(0.0) - px[2]) * a;
            });
    } else {
        let neg = -a;
        image
            .rgb
            .par_chunks_exact_mut(3)
            .enumerate()
            .for_each(|(i, px)| {
                let ti = t[i];
                let t_add = (1.0 - ti * neg * 0.5).clamp(0.0, 1.0);
                px[0] = atm[0] * (1.0 - t_add) + px[0] * t_add;
                px[1] = atm[1] * (1.0 - t_add) + px[1] * t_add;
                px[2] = atm[2] * (1.0 - t_add) + px[2] * t_add;
            });
    }
}
