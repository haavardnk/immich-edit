use crate::cpu::scratch::Scratch;
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

fn box_mean(src: &[f32], w: usize, h: usize, r: usize) -> Scratch {
    let mut tmp = Scratch::take_uninit(w * h);
    let mut out = Scratch::take_uninit(w * h);
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

fn min_filter(src: &[f32], w: usize, h: usize, r: usize) -> Scratch {
    let mut tmp = Scratch::take_uninit(w * h);
    let mut out = Scratch::take_uninit(w * h);
    min_filter_h(src, &mut tmp, w, h, r);
    min_filter_v(&tmp, &mut out, w, h, r);
    out
}

fn dark_channel_per_pixel(rgb: &[f32], w: usize, h: usize) -> Scratch {
    let mut out = Scratch::take_uninit(w * h);
    out.par_iter_mut().enumerate().for_each(|(i, v)| {
        let r = rgb[i * 3].clamp(0.0, 1.0);
        let g = rgb[i * 3 + 1].clamp(0.0, 1.0);
        let b = rgb[i * 3 + 2].clamp(0.0, 1.0);
        *v = r.min(g).min(b);
    });
    out
}

pub fn estimate_atmosphere(rgb: &[f32], dp: &[f32], w: usize, h: usize) -> [f32; 3] {
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

fn guided_filter(guide: &[f32], p: &[f32], w: usize, h: usize, r: usize, eps: f32) -> Scratch {
    let n = w * h;
    let mean_i = box_mean(guide, w, h, r);
    let mean_p = box_mean(p, w, h, r);
    let mut ii = Scratch::take_uninit(n);
    ii.par_iter_mut()
        .zip(guide.par_iter())
        .for_each(|(d, x)| *d = x * x);
    let mut ip = Scratch::take_uninit(n);
    ip.par_iter_mut()
        .zip(guide.par_iter().zip(p.par_iter()))
        .for_each(|(d, (a, b))| *d = a * b);
    let corr_i = box_mean(&ii, w, h, r);
    drop(ii);
    let corr_ip = box_mean(&ip, w, h, r);
    drop(ip);
    let mut a_coef = Scratch::take_uninit(n);
    a_coef
        .par_iter_mut()
        .zip(
            corr_ip
                .par_iter()
                .zip(corr_i.par_iter())
                .zip(mean_i.par_iter().zip(mean_p.par_iter())),
        )
        .for_each(|(d, ((cip, ci), (mi, mp)))| {
            let var_i = ci - mi * mi;
            let cov_ip = cip - mi * mp;
            *d = cov_ip / (var_i + eps);
        });
    drop(corr_i);
    drop(corr_ip);
    let mut b_coef = Scratch::take_uninit(n);
    b_coef
        .par_iter_mut()
        .zip(
            a_coef
                .par_iter()
                .zip(mean_i.par_iter().zip(mean_p.par_iter())),
        )
        .for_each(|(d, (a, (mi, mp)))| *d = mp - a * mi);
    let mean_a = box_mean(&a_coef, w, h, r);
    drop(a_coef);
    let mean_b = box_mean(&b_coef, w, h, r);
    drop(b_coef);
    let mut out = Scratch::take_uninit(n);
    out.par_iter_mut()
        .zip(
            mean_a
                .par_iter()
                .zip(mean_b.par_iter().zip(guide.par_iter())),
        )
        .for_each(|(d, (ma, (mb, g)))| *d = ma * g + mb);
    out
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
    drop(d0);
    let atm = estimate_atmosphere(&image.rgb, &dp, w, h);
    drop(dp);
    let n = w * h;
    let mut dn = Scratch::take_uninit(n);
    dn.par_iter_mut().enumerate().for_each(|(i, v)| {
        let r = (image.rgb[i * 3] / atm[0]).clamp(0.0, 1.0);
        let g = (image.rgb[i * 3 + 1] / atm[1]).clamp(0.0, 1.0);
        let b = (image.rgb[i * 3 + 2] / atm[2]).clamp(0.0, 1.0);
        *v = r.min(g).min(b);
    });
    let dn_patch = min_filter(&dn, w, h, r_patch);
    drop(dn);
    let mut t_raw = Scratch::take_uninit(n);
    t_raw
        .par_iter_mut()
        .zip(dn_patch.par_iter())
        .for_each(|(d, s)| *d = (1.0 - 0.95 * s).clamp(0.0, 1.0));
    drop(dn_patch);
    let mut guide = Scratch::take_uninit(n);
    guide.par_iter_mut().enumerate().for_each(|(i, v)| {
        *v = luma(
            image.rgb[i * 3].clamp(0.0, 1.0),
            image.rgb[i * 3 + 1].clamp(0.0, 1.0),
            image.rgb[i * 3 + 2].clamp(0.0, 1.0),
        );
    });
    let t = guided_filter(&guide, &t_raw, w, h, r_gf, 1e-3);
    drop(guide);
    drop(t_raw);
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
