use crate::edits::HSL_BANDS;
use crate::math::{hue_dist, linear_srgb_to_oklab, oklab_to_linear_srgb, smoothstep};
use crate::ops::hsl::{
    HSL_BAND_CENTERS_DEG, HSL_BAND_SIGMA_DEG, HSL_LUM_EV, HSL_MIN_SAT, HSL_SAT_GATE_HI,
    HSL_SAT_GATE_LO,
};

#[inline(always)]
fn hue_and_sat(r: f32, g: f32, b: f32) -> (f32, f32) {
    let k = r.max(g).max(b).max(1.0);
    let r = r / k;
    let g = g / k;
    let b = b / k;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    if d < 1e-6 {
        return (0.0, 0.0);
    }
    let s = if max + min > 1.0 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s)
}

#[inline(always)]
fn band_weights(h_deg: f32) -> [f32; HSL_BANDS] {
    let mut w = [0.0f32; HSL_BANDS];
    let sigma2 = HSL_BAND_SIGMA_DEG * HSL_BAND_SIGMA_DEG;
    for i in 0..HSL_BANDS {
        let d = hue_dist(h_deg, HSL_BAND_CENTERS_DEG[i]);
        w[i] = crate::math::fast::exp(-(d * d) / (2.0 * sigma2));
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
    let (h, s) = hue_and_sat(r.max(0.0), g.max(0.0), b.max(0.0));
    if s < HSL_MIN_SAT {
        return;
    }
    let w = band_weights(h);
    let gate = smoothstep(HSL_SAT_GATE_LO, HSL_SAT_GATE_HI, s);
    let mut hue_delta = 0.0f32;
    let mut sat_delta = 0.0f32;
    let mut lum_delta = 0.0f32;
    for i in 0..HSL_BANDS {
        hue_delta += hue_shifts[i] * w[i];
        sat_delta += sat_gains[i] * w[i];
        lum_delta += lum_gains[i] * w[i];
    }
    let (sin, cos) = (hue_delta * gate).to_radians().sin_cos();
    let chroma = (1.0 + sat_delta * gate).max(0.0);
    let [l, a, bb] = linear_srgb_to_oklab([*r, *g, *b]);
    let rotated = [
        l,
        (a * cos - bb * sin) * chroma,
        (a * sin + bb * cos) * chroma,
    ];
    let gain = crate::math::fast::exp2(lum_delta * gate * HSL_LUM_EV);
    let [nr, ng, nb] = oklab_to_linear_srgb(rotated);
    *r = nr * gain;
    *g = ng * gain;
    *b = nb * gain;
}
