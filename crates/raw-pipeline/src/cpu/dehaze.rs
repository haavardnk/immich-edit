use crate::cpu::scratch::Scratch;
use crate::math::luma;
use crate::ops::LinearImage;
use rayon::prelude::*;

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

pub fn patch_radius(w: usize, h: usize) -> usize {
    let min_dim = w.min(h);
    (min_dim / 200).max(8).min((min_dim / 2).max(1))
}

pub fn guided_scale(w: usize, h: usize) -> usize {
    if w.min(h) >= 512 { 4 } else { 1 }
}

fn mip_halve(rgb: &[f32], w: usize, h: usize) -> (Vec<f32>, usize, usize) {
    let lw = (w / 2).max(1);
    let lh = (h / 2).max(1);
    let mut out = vec![0.0f32; lw * lh * 3];
    out.par_chunks_exact_mut(lw * 3)
        .enumerate()
        .for_each(|(ly, row)| {
            let sy = ly * 2;
            for (lx, px) in row.chunks_exact_mut(3).enumerate() {
                let sx = lx * 2;
                for (c, v) in px.iter_mut().enumerate() {
                    let mut acc = rgb[(sy * w + sx) * 3 + c];
                    if sx + 1 < w {
                        acc += rgb[(sy * w + sx + 1) * 3 + c];
                    }
                    if sy + 1 < h {
                        acc += rgb[((sy + 1) * w + sx) * 3 + c];
                    }
                    if sx + 1 < w && sy + 1 < h {
                        acc += rgb[((sy + 1) * w + sx + 1) * 3 + c];
                    }
                    *v = acc * 0.25;
                }
            }
        });
    (out, lw, lh)
}

fn bilinear_downsample(rgb: &[f32], w: usize, h: usize, lw: usize, lh: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; lw * lh * 3];
    let maxx = w as isize - 1;
    let maxy = h as isize - 1;
    out.par_chunks_exact_mut(lw * 3)
        .enumerate()
        .for_each(|(ly, row)| {
            let sy = (ly as f32 + 0.5) / lh as f32 * h as f32 - 0.5;
            let by = sy.floor();
            let fy = sy - by;
            let ya = (by as isize).clamp(0, maxy) as usize;
            let yb = (by as isize + 1).clamp(0, maxy) as usize;
            for (lx, px) in row.chunks_exact_mut(3).enumerate() {
                let sx = (lx as f32 + 0.5) / lw as f32 * w as f32 - 0.5;
                let bx = sx.floor();
                let fx = sx - bx;
                let xa = (bx as isize).clamp(0, maxx) as usize;
                let xb = (bx as isize + 1).clamp(0, maxx) as usize;
                let i00 = (ya * w + xa) * 3;
                let i10 = (ya * w + xb) * 3;
                let i01 = (yb * w + xa) * 3;
                let i11 = (yb * w + xb) * 3;
                for (c, v) in px.iter_mut().enumerate() {
                    let top = rgb[i00 + c] + (rgb[i10 + c] - rgb[i00 + c]) * fx;
                    let bot = rgb[i01 + c] + (rgb[i11 + c] - rgb[i01 + c]) * fx;
                    *v = top + (bot - top) * fy;
                }
            }
        });
    out
}

pub fn atmosphere_for_render(rgb: &[f32], w: usize, h: usize) -> [f32; 3] {
    let max_dim = w.max(h);
    let levels = if max_dim <= 256 {
        0
    } else {
        (max_dim as f32 / 256.0).log2().ceil() as u32
    };
    let mut reduced: Option<Vec<f32>> = None;
    let mut cw = w;
    let mut ch = h;
    for _ in 0..levels {
        if cw <= 1 && ch <= 1 {
            break;
        }
        let (next, nw, nh) = match reduced.as_deref() {
            Some(prev) => mip_halve(prev, cw, ch),
            None => mip_halve(rgb, cw, ch),
        };
        reduced = Some(next);
        cw = nw;
        ch = nh;
    }
    match reduced {
        Some(v) => atmosphere_from_rgb(&v, cw, ch),
        None => atmosphere_from_rgb(rgb, w, h),
    }
}

pub fn atmosphere_from_rgb(rgb: &[f32], w: usize, h: usize) -> [f32; 3] {
    let d0 = dark_channel_per_pixel(rgb, w, h);
    let dp = min_filter(&d0, w, h, patch_radius(w, h));
    estimate_atmosphere(rgb, &dp, w, h)
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

fn guided_coeffs(
    guide: &[f32],
    p: &[f32],
    w: usize,
    h: usize,
    r: usize,
    eps: f32,
) -> (Scratch, Scratch) {
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
            let var_i = (ci - mi * mi).max(0.0);
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
    (mean_a, mean_b)
}

fn sample_ab(mean_a: &[f32], mean_b: &[f32], lw: usize, lh: usize, u: f32, v: f32) -> (f32, f32) {
    let sx = u * lw as f32 - 0.5;
    let sy = v * lh as f32 - 0.5;
    let bx = sx.floor();
    let by = sy.floor();
    let fx = sx - bx;
    let fy = sy - by;
    let maxx = lw as isize - 1;
    let maxy = lh as isize - 1;
    let xa = (bx as isize).clamp(0, maxx) as usize;
    let xb = (bx as isize + 1).clamp(0, maxx) as usize;
    let ya = (by as isize).clamp(0, maxy) as usize;
    let yb = (by as isize + 1).clamp(0, maxy) as usize;
    let mix = |c00: f32, c10: f32, c01: f32, c11: f32| {
        let top = c00 + (c10 - c00) * fx;
        let bot = c01 + (c11 - c01) * fx;
        top + (bot - top) * fy
    };
    let i00 = ya * lw + xa;
    let i10 = ya * lw + xb;
    let i01 = yb * lw + xa;
    let i11 = yb * lw + xb;
    (
        mix(mean_a[i00], mean_a[i10], mean_a[i01], mean_a[i11]),
        mix(mean_b[i00], mean_b[i10], mean_b[i01], mean_b[i11]),
    )
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
    let half_min = (min_dim / 2).max(1);
    let scale = guided_scale(w, h);
    let lw = (w / scale).max(1);
    let lh = (h / scale).max(1);
    let r_patch = (patch_radius(w, h) / scale).max(2);
    let r_gf = (((min_dim / 50).max(16).min(half_min)) / scale).max(4);
    let atm = atmosphere_for_render(&image.rgb, w, h);
    let lo = if scale == 1 {
        image.rgb.clone()
    } else {
        bilinear_downsample(&image.rgb, w, h, lw, lh)
    };
    let n = lw * lh;
    let mut dn = Scratch::take_uninit(n);
    dn.par_iter_mut().enumerate().for_each(|(i, v)| {
        let r = (lo[i * 3] / atm[0]).clamp(0.0, 1.0);
        let g = (lo[i * 3 + 1] / atm[1]).clamp(0.0, 1.0);
        let b = (lo[i * 3 + 2] / atm[2]).clamp(0.0, 1.0);
        *v = r.min(g).min(b);
    });
    let dn_patch = min_filter(&dn, lw, lh, r_patch);
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
            lo[i * 3].clamp(0.0, 1.0),
            lo[i * 3 + 1].clamp(0.0, 1.0),
            lo[i * 3 + 2].clamp(0.0, 1.0),
        );
    });
    let (mean_a, mean_b) = guided_coeffs(&guide, &t_raw, lw, lh, r_gf, 1e-3);
    drop(guide);
    drop(t_raw);
    drop(lo);
    let transmission = |x: usize, y: usize, px: &[f32]| -> f32 {
        let u = (x as f32 + 0.5) / w as f32;
        let v = (y as f32 + 0.5) / h as f32;
        let (ca, cb) = sample_ab(&mean_a, &mean_b, lw, lh, u, v);
        let g = luma(
            px[0].clamp(0.0, 1.0),
            px[1].clamp(0.0, 1.0),
            px[2].clamp(0.0, 1.0),
        );
        (ca * g + cb).clamp(0.0, 1.0)
    };
    if a > 0.0 {
        image
            .rgb
            .par_chunks_exact_mut(w * 3)
            .enumerate()
            .for_each(|(y, row)| {
                for (x, px) in row.chunks_exact_mut(3).enumerate() {
                    let ti = transmission(x, y, px).max(0.16);
                    let jr = (px[0] - atm[0]) / ti + atm[0];
                    let jg = (px[1] - atm[1]) / ti + atm[1];
                    let jb = (px[2] - atm[2]) / ti + atm[2];
                    px[0] = px[0] + (jr.max(0.0) - px[0]) * a;
                    px[1] = px[1] + (jg.max(0.0) - px[1]) * a;
                    px[2] = px[2] + (jb.max(0.0) - px[2]) * a;
                }
            });
    } else {
        let neg = -a;
        image
            .rgb
            .par_chunks_exact_mut(w * 3)
            .enumerate()
            .for_each(|(y, row)| {
                for (x, px) in row.chunks_exact_mut(3).enumerate() {
                    let ti = transmission(x, y, px);
                    let t_add = (1.0 - ti * neg * 0.5).clamp(0.0, 1.0);
                    px[0] = atm[0] * (1.0 - t_add) + px[0] * t_add;
                    px[1] = atm[1] * (1.0 - t_add) + px[1] * t_add;
                    px[2] = atm[2] * (1.0 - t_add) + px[2] * t_add;
                }
            });
    }
}
