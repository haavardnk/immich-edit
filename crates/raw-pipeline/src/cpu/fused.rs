use crate::cpu::presence_pyramid::LumaPyramid;
use crate::edits::{CURVE_LUT_SIZE, HSL_BANDS};
use crate::ops::LinearImage;
use rayon::prelude::*;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum CpuFusedOp {
    WhiteBalance {
        coeffs: [f32; 3],
    },
    ColorMatrix {
        m: [[f32; 3]; 3],
    },
    Exposure {
        factor: f32,
    },
    Contrast {
        factor: f32,
    },
    Saturation {
        factor: f32,
    },
    Vibrance {
        amount: f32,
    },
    ToneRegions {
        hl: f32,
        sh: f32,
        bk: f32,
        wh: f32,
    },
    Curves {
        lut: [f32; CURVE_LUT_SIZE],
    },
    Hsl {
        hue_shifts: [f32; HSL_BANDS],
        sat_gains: [f32; HSL_BANDS],
        lum_gains: [f32; HSL_BANDS],
    },
    ColorGrade {
        s_off: [f32; 3],
        s_lum: f32,
        m_off: [f32; 3],
        m_lum: f32,
        h_off: [f32; 3],
        h_lum: f32,
        g_off: [f32; 3],
        g_lum: f32,
        balance: f32,
        blend: f32,
    },
    Presence {
        pyramid: Arc<LumaPyramid>,
        texture: f32,
        clarity: f32,
        dehaze: f32,
        mip_texture: u32,
        mip_clarity: u32,
        mip_dehaze: u32,
    },
}

#[derive(Default, Clone, Debug)]
pub struct FusedSegment {
    pub ops: Vec<CpuFusedOp>,
}

impl FusedSegment {
    pub fn push(&mut self, op: CpuFusedOp) {
        self.ops.push(op);
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    pub fn clear(&mut self) {
        self.ops.clear();
    }
}

#[inline(always)]
fn apply_one(op: &CpuFusedOp, i: usize, img_w: usize, r: &mut f32, g: &mut f32, b: &mut f32) {
    match op {
        CpuFusedOp::WhiteBalance { coeffs } => {
            *r *= coeffs[0];
            *g *= coeffs[1];
            *b *= coeffs[2];
        }
        CpuFusedOp::ColorMatrix { m } => {
            let nr = m[0][0] * *r + m[0][1] * *g + m[0][2] * *b;
            let ng = m[1][0] * *r + m[1][1] * *g + m[1][2] * *b;
            let nb = m[2][0] * *r + m[2][1] * *g + m[2][2] * *b;
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::Exposure { factor } => {
            *r *= *factor;
            *g *= *factor;
            *b *= *factor;
        }
        CpuFusedOp::Contrast { factor } => {
            *r = (*r - 0.5) * *factor + 0.5;
            *g = (*g - 0.5) * *factor + 0.5;
            *b = (*b - 0.5) * *factor + 0.5;
        }
        CpuFusedOp::Saturation { factor } => {
            let luma = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
            *r = luma + (*r - luma) * *factor;
            *g = luma + (*g - luma) * *factor;
            *b = luma + (*b - luma) * *factor;
        }
        CpuFusedOp::Vibrance { amount } => {
            let mx = r.max(*g).max(*b);
            let mn = r.min(*g).min(*b);
            let sat = (mx - mn).clamp(0.0, 1.0);
            let factor = 1.0 + *amount * (1.0 - sat);
            let luma = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
            *r = luma + (*r - luma) * factor;
            *g = luma + (*g - luma) * factor;
            *b = luma + (*b - luma) * factor;
        }
        CpuFusedOp::ToneRegions { hl, sh, bk, wh } => {
            *r = tone_zone(*r, *hl, *sh, *bk, *wh);
            *g = tone_zone(*g, *hl, *sh, *bk, *wh);
            *b = tone_zone(*b, *hl, *sh, *bk, *wh);
        }
        CpuFusedOp::Curves { lut } => {
            *r = curves_sample(lut, *r);
            *g = curves_sample(lut, *g);
            *b = curves_sample(lut, *b);
        }
        CpuFusedOp::Hsl {
            hue_shifts,
            sat_gains,
            lum_gains,
        } => {
            let (h, s, l) = rgb_to_hsl(r.clamp(0.0, 2.0), g.clamp(0.0, 2.0), b.clamp(0.0, 2.0));
            if s < 1e-4 {
                return;
            }
            let w = band_weights(h);
            let mut hue_delta = 0.0f32;
            let mut sat_delta = 0.0f32;
            let mut lum_delta = 0.0f32;
            for i in 0..HSL_BANDS {
                hue_delta += hue_shifts[i] * w[i];
                sat_delta += sat_gains[i] * w[i];
                lum_delta += lum_gains[i] * w[i];
            }
            let new_h = h + hue_delta;
            let new_s = (s * (1.0 + sat_delta)).clamp(0.0, 1.0);
            let new_l = (l + lum_delta * 0.3).clamp(0.0, 1.0);
            let (nr, ng, nb) = hsl_to_rgb(new_h, new_s, new_l);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::ColorGrade {
            s_off,
            s_lum,
            m_off,
            m_lum,
            h_off,
            h_lum,
            g_off,
            g_lum,
            balance,
            blend,
        } => {
            const STRENGTH: f32 = 0.5;
            let y = (0.2126 * *r + 0.7152 * *g + 0.0722 * *b).clamp(0.0, 1.0);
            let (ws, wm, wh) = cg_weights(y, *balance, *blend);
            let or = (ws * s_off[0] + wm * m_off[0] + wh * h_off[0] + g_off[0]) * STRENGTH;
            let og = (ws * s_off[1] + wm * m_off[1] + wh * h_off[1] + g_off[1]) * STRENGTH;
            let ob = (ws * s_off[2] + wm * m_off[2] + wh * h_off[2] + g_off[2]) * STRENGTH;
            let lum = (ws * *s_lum + wm * *m_lum + wh * *h_lum + *g_lum) * STRENGTH;
            *r = (*r + or + lum).max(0.0);
            *g = (*g + og + lum).max(0.0);
            *b = (*b + ob + lum).max(0.0);
        }
        CpuFusedOp::Presence {
            pyramid,
            texture,
            clarity,
            dehaze,
            mip_texture,
            mip_clarity,
            mip_dehaze,
        } => {
            let x = i % img_w;
            let y = i / img_w;
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let y0 = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
            let mut delta = 0.0f32;
            if *texture != 0.0 {
                let bl = pyramid.sample(*mip_texture, fx, fy);
                delta += *texture * (y0 - bl);
            }
            if *clarity != 0.0 {
                let bl = pyramid.sample(*mip_clarity, fx, fy);
                let mt = 1.0 - (2.0 * y0 - 1.0).abs();
                delta += *clarity * mt * (y0 - bl);
            }
            if *dehaze != 0.0 {
                let bl = pyramid.sample(*mip_dehaze, fx, fy);
                delta += *dehaze * (y0 - bl);
            }
            let goal = (y0 + delta).max(0.0);
            if y0 <= 1e-5 {
                *r = goal;
                *g = goal;
                *b = goal;
            } else {
                let scale = goal / y0;
                *r = (*r * scale).max(0.0);
                *g = (*g * scale).max(0.0);
                *b = (*b * scale).max(0.0);
            }
        }
    }
}

#[inline(always)]
fn curves_sample(lut: &[f32; CURVE_LUT_SIZE], v: f32) -> f32 {
    let x = v.clamp(0.0, 1.0) * (CURVE_LUT_SIZE - 1) as f32;
    let idx = (x as usize).min(CURVE_LUT_SIZE - 2);
    let frac = x - idx as f32;
    lut[idx] * (1.0 - frac) + lut[idx + 1] * frac
}

const BAND_CENTERS_DEG: [f32; HSL_BANDS] = [0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 270.0, 300.0];
const BAND_SIGMA_DEG: f32 = 25.0;

#[inline(always)]
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) * 0.5;
    let d = max - min;
    if d < 1e-6 {
        return (0.0, 0.0, l);
    }
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let mut h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h *= 60.0;
    (h, s, l)
}

#[inline(always)]
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

#[inline(always)]
fn hsl_to_rgb(h_deg: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s <= 0.0 {
        return (l, l, l);
    }
    let h = (h_deg.rem_euclid(360.0)) / 360.0;
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

#[inline(always)]
fn hue_dist(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[inline(always)]
fn band_weights(h_deg: f32) -> [f32; HSL_BANDS] {
    let mut w = [0.0f32; HSL_BANDS];
    let sigma2 = BAND_SIGMA_DEG * BAND_SIGMA_DEG;
    for i in 0..HSL_BANDS {
        let d = hue_dist(h_deg, BAND_CENTERS_DEG[i]);
        w[i] = (-(d * d) / (2.0 * sigma2)).exp();
    }
    w
}

#[inline(always)]
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline(always)]
fn cg_weights(y: f32, balance: f32, blend: f32) -> (f32, f32, f32) {
    let pivot = 0.5 + 0.3 * balance;
    let feather = 0.15 + 0.25 * blend;
    let s_hi = (pivot + feather * 0.5).clamp(0.001, 0.999);
    let s_lo = (pivot - feather - feather * 0.5).clamp(0.0, s_hi - 0.001);
    let h_lo = (pivot - feather * 0.5).clamp(0.001, 0.999);
    let h_hi = (pivot + feather + feather * 0.5).clamp(h_lo + 0.001, 1.0);
    let shadow = 1.0 - smoothstep(s_lo, s_hi, y);
    let highlight = smoothstep(h_lo, h_hi, y);
    let mid = (1.0 - shadow - highlight).max(0.0);
    (shadow, mid, highlight)
}

#[inline(always)]
fn tone_zone(x: f32, hl: f32, sh: f32, bk: f32, wh: f32) -> f32 {
    let xc = x.clamp(0.0, 2.0);
    let xm = xc.min(1.0);
    let w_bk = ((0.2 - xm) / 0.2).clamp(0.0, 1.0);
    let w_sh = (1.0 - (xm - 0.25).abs() / 0.4).clamp(0.0, 1.0);
    let w_hl = (1.0 - (xm - 0.75).abs() / 0.4).clamp(0.0, 1.0);
    let w_wh = ((xm - 0.8) / 0.2).clamp(0.0, 1.0);
    let delta = hl * w_hl * (1.0 - xc).max(-1.0) * 0.5
        + sh * w_sh * xc * 0.5
        + bk * w_bk * 0.2
        + wh * w_wh * (1.0 - xc).max(-1.0) * 0.5;
    xc + delta
}

pub fn apply_segment(image: &mut LinearImage, segment: &FusedSegment) {
    if segment.is_empty() {
        return;
    }
    let ops = segment.ops.as_slice();
    let img_w = image.width;
    image
        .rgb
        .par_chunks_exact_mut(3)
        .enumerate()
        .for_each(|(i, px)| {
            let mut r = px[0];
            let mut g = px[1];
            let mut b = px[2];
            for op in ops {
                apply_one(op, i, img_w, &mut r, &mut g, &mut b);
            }
            px[0] = r;
            px[1] = g;
            px[2] = b;
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::identity_3x3;
    use crate::edits::{BasicEdits, Edits};
    use crate::ops::EditOperator;
    use crate::ops::{OpContext, color_matrix, exposure, white_balance};

    fn ramp_image(w: usize, h: usize) -> Vec<f32> {
        let mut v = Vec::with_capacity(w * h * 3);
        for i in 0..(w * h) {
            let t = i as f32 / (w * h) as f32;
            v.push(t * 0.7);
            v.push(t * 0.5 + 0.1);
            v.push((1.0 - t) * 0.6);
        }
        v
    }

    fn max_abs_diff(a: &[f32], b: &[f32]) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max)
    }

    #[test]
    fn fused_matches_legacy_wb_cm_exposure() {
        let w = 16;
        let h = 16;
        let buf = ramp_image(w, h);
        let ctx = OpContext {
            wb_coeffs: [2.1, 1.0, 1.45, 1.0],
            cam_to_srgb: [[1.2, -0.1, -0.1], [-0.05, 1.05, 0.0], [0.0, -0.2, 1.2]],
            is_raw: true,
        };
        let edits = Edits {
            basic: BasicEdits {
                exposure_ev: 0.7,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut legacy = LinearImage::new(buf.clone(), w, h);
        white_balance::WhiteBalanceOp
            .apply_cpu(&mut legacy, &ctx, &edits)
            .unwrap();
        color_matrix::ColorMatrixOp
            .apply_cpu(&mut legacy, &ctx, &edits)
            .unwrap();
        exposure::ExposureOp
            .apply_cpu(&mut legacy, &ctx, &edits)
            .unwrap();

        let mut fused = LinearImage::new(buf, w, h);
        let mut seg = FusedSegment::default();
        seg.push(
            white_balance::WhiteBalanceOp
                .cpu_fused(&edits, &ctx)
                .unwrap(),
        );
        seg.push(color_matrix::ColorMatrixOp.cpu_fused(&edits, &ctx).unwrap());
        seg.push(exposure::ExposureOp.cpu_fused(&edits, &ctx).unwrap());
        apply_segment(&mut fused, &seg);

        let d = max_abs_diff(&legacy.rgb, &fused.rgb);
        if d > 1e-5 {
            panic!("fused vs legacy diff too high: {d}");
        }
    }

    #[test]
    fn fused_skips_color_matrix_when_not_raw() {
        let ctx = OpContext {
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            cam_to_srgb: identity_3x3(),
            is_raw: false,
        };
        let edits = Edits::default();
        if color_matrix::ColorMatrixOp
            .cpu_fused(&edits, &ctx)
            .is_some()
        {
            panic!("expected None when not raw");
        }
    }
}
