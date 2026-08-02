use crate::edits::HSL_BANDS;
use crate::math::{hue_dist, smoothstep};

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
pub fn apply_hsl(
    hue_shifts: &[f32; HSL_BANDS],
    sat_gains: &[f32; HSL_BANDS],
    lum_gains: &[f32; HSL_BANDS],
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) {
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
