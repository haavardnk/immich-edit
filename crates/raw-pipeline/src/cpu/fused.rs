use crate::edits::HSL_BANDS;
use crate::ops::LinearImage;
use crate::ops::curves::{CurveLuts, apply_curves_pixel};
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
    Brightness {
        amount: f32,
    },
    Contrast {
        s: f32,
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
        wh_gain: f32,
        shadows_blur: Option<Arc<Vec<f32>>>,
    },
    Curves {
        luts: Box<CurveLuts>,
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
        texture: f32,
        clarity: f32,
        dehaze: f32,
        texture_blur: Option<Arc<Vec<f32>>>,
        clarity_blur: Option<Arc<Vec<f32>>>,
        dehaze_blur: Option<Arc<Vec<f32>>>,
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
pub fn apply_one(op: &CpuFusedOp, i: usize, r: &mut f32, g: &mut f32, b: &mut f32) {
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
        CpuFusedOp::Brightness { amount } => {
            let (nr, ng, nb) = crate::ops::brightness::apply_brightness_rgb(*r, *g, *b, *amount);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::Contrast { s } => {
            *r = crate::ops::contrast::apply_perceptual_contrast(*r, *s);
            *g = crate::ops::contrast::apply_perceptual_contrast(*g, *s);
            *b = crate::ops::contrast::apply_perceptual_contrast(*b, *s);
        }
        CpuFusedOp::Saturation { factor } => {
            let luma = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
            *r = luma + (*r - luma) * *factor;
            *g = luma + (*g - luma) * *factor;
            *b = luma + (*b - luma) * *factor;
        }
        CpuFusedOp::Vibrance { amount } => {
            let (nr, ng, nb) = crate::ops::vibrance::apply_vibrance_rgb(*r, *g, *b, *amount);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::ToneRegions {
            hl,
            sh,
            bk,
            wh_gain,
            shadows_blur,
        } => {
            *r *= *wh_gain;
            *g *= *wh_gain;
            *b *= *wh_gain;
            if *sh != 0.0 {
                let luma = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
                let blur_l = shadows_blur.as_ref().map(|buf| buf[i]).unwrap_or(luma);
                let mult = crate::ops::tone_regions::shadows_mult(luma, blur_l, *sh);
                *r *= mult;
                *g *= mult;
                *b *= mult;
            }
            let (nr, ng, nb) =
                crate::ops::tone_regions::apply_tone_regions_rgb(*r, *g, *b, *hl, *bk);
            *r = nr;
            *g = ng;
            *b = nb;
        }
        CpuFusedOp::Curves { luts } => {
            apply_curves_pixel(luts.as_ref(), r, g, b);
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
            let gate = smoothstep(0.05, 0.20, s);
            let mut hue_delta = 0.0f32;
            let mut sat_delta = 0.0f32;
            let mut lum_delta = 0.0f32;
            for i in 0..HSL_BANDS {
                hue_delta += hue_shifts[i] * w[i];
                sat_delta += sat_gains[i] * w[i];
                lum_delta += lum_gains[i] * w[i];
            }
            hue_delta *= gate;
            sat_delta *= gate;
            lum_delta *= gate;
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
            texture,
            clarity,
            dehaze,
            texture_blur,
            clarity_blur,
            dehaze_blur,
        } => {
            let y0 = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
            let y0c = y0.max(1e-5);
            let mut log_gain = 0.0f32;
            if let Some(buf) = texture_blur {
                log_gain += *texture * (y0c / buf[i].max(1e-5)).log2();
            }
            if let Some(buf) = clarity_blur {
                let mt = smoothstep(0.0, 0.1, y0)
                    * (1.0 - smoothstep(0.9, 1.0, y0))
                    * (1.0 - (2.0 * y0 - 1.0).abs()).max(0.0);
                let ratio = (y0c / buf[i].max(1e-5)).log2();
                let gate = smoothstep(0.015, 0.12, ratio.abs());
                log_gain += *clarity * mt * gate * ratio;
            }
            let mut new_y = y0 * log_gain.exp2();
            if let Some(buf) = dehaze_blur {
                new_y += *dehaze * (y0 - buf[i]);
            }
            let goal = new_y.max(0.0);
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
    let sum: f32 = w.iter().sum();
    if sum > 1.0 {
        for v in &mut w {
            *v /= sum;
        }
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

pub fn apply_segment(image: &mut LinearImage, segment: &FusedSegment) {
    if segment.is_empty() {
        return;
    }
    let ops = segment.ops.as_slice();
    let img_w = image.width;
    let row_floats = img_w * 3;
    image
        .rgb
        .par_chunks_exact_mut(row_floats)
        .enumerate()
        .for_each(|(y, row)| {
            let row_base = y * img_w;
            for (x, px) in row.chunks_exact_mut(3).enumerate() {
                let i = row_base + x;
                let mut r = px[0];
                let mut g = px[1];
                let mut b = px[2];
                for op in ops {
                    apply_one(op, i, &mut r, &mut g, &mut b);
                }
                px[0] = r;
                px[1] = g;
                px[2] = b;
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::color::identity_3x3;
    use crate::edits::Edits;
    use crate::ops::EditOperator;
    use crate::ops::{OpContext, OpScratch, RenderContext, color_matrix};

    #[test]
    fn fused_skips_color_matrix_when_not_raw() {
        let ctx = OpContext {
            render: RenderContext {
                wb_coeffs: [1.0, 1.0, 1.0, 1.0],
                cam_to_srgb: identity_3x3(),
                is_raw: false,
                preview_mode: crate::frame::PreviewMode::None,
            },
            scratch: OpScratch { shadows_blur: None },
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
