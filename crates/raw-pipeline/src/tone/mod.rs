pub mod shared;
pub mod wgsl;

use crate::frame::OutputColorSpace;
use shared::{
    LUMA_B, LUMA_G, LUMA_R, OETF_LUT_SIZE, S_CURVE_BLEND, SRGB_OETF_GAMMA, SRGB_OETF_GAMMA_OFFSET,
    SRGB_OETF_GAMMA_SCALE, SRGB_OETF_LINEAR_CUTOFF, SRGB_OETF_LINEAR_SLOPE, TONE_SHOULDER_KNEE,
};

fn oetf_lut() -> &'static [f32; OETF_LUT_SIZE + 1] {
    static LUT: std::sync::OnceLock<[f32; OETF_LUT_SIZE + 1]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0f32; OETF_LUT_SIZE + 1];
        for (i, slot) in t.iter_mut().enumerate() {
            let v = i as f32 / OETF_LUT_SIZE as f32;
            *slot = srgb_oetf_scalar(v);
        }
        t
    })
}

pub fn srgb_oetf(v: f32) -> f32 {
    let lut = oetf_lut();
    let scaled = v.clamp(0.0, 1.0) * OETF_LUT_SIZE as f32;
    let idx = scaled as usize;
    let frac = scaled - idx as f32;
    let lo = lut[idx];
    let hi = lut[(idx + 1).min(OETF_LUT_SIZE)];
    lo + (hi - lo) * frac
}

pub fn srgb_oetf_scalar(v: f32) -> f32 {
    if v <= SRGB_OETF_LINEAR_CUTOFF {
        SRGB_OETF_LINEAR_SLOPE * v
    } else {
        SRGB_OETF_GAMMA_SCALE * v.powf(SRGB_OETF_GAMMA) - SRGB_OETF_GAMMA_OFFSET
    }
}

fn highlight_shoulder(x: f32) -> f32 {
    let k = TONE_SHOULDER_KNEE;
    if x <= k {
        return x;
    }
    let headroom = 1.0 - k;
    1.0 - headroom * (-(x - k) / headroom).exp()
}

fn display_encode(v: f32) -> f32 {
    let c = v.clamp(0.0, 1.0);
    let srgb = srgb_oetf(c);
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    srgb + (s - srgb) * S_CURVE_BLEND
}

pub fn default_scalar(v: f32) -> f32 {
    display_encode(highlight_shoulder(v.max(0.0)))
}

fn luma(rgb: [f32; 3]) -> f32 {
    LUMA_R * rgb[0] + LUMA_G * rgb[1] + LUMA_B * rgb[2]
}

fn project_to_gamut(rgb: [f32; 3], neutral: f32) -> [f32; 3] {
    let mut out = rgb;
    let mn = out[0].min(out[1]).min(out[2]);
    if mn < 0.0 {
        let t = (-mn / (neutral - mn)).clamp(0.0, 1.0);
        out = [
            out[0] + (neutral - out[0]) * t,
            out[1] + (neutral - out[1]) * t,
            out[2] + (neutral - out[2]) * t,
        ];
    }
    let mx = out[0].max(out[1]).max(out[2]);
    if mx > 1.0 {
        let t = ((mx - 1.0) / (mx - neutral)).clamp(0.0, 1.0);
        out = [
            out[0] + (neutral - out[0]) * t,
            out[1] + (neutral - out[1]) * t,
            out[2] + (neutral - out[2]) * t,
        ];
    }
    out
}

fn to_output_space(rgb: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    match cs {
        OutputColorSpace::SRgb => rgb,
        OutputColorSpace::DisplayP3 => crate::color::srgb_lin_to_display_p3(rgb),
    }
}

fn tone_map_luma_and_project_cs(rgb: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    let y = luma(rgb);
    if y <= 1e-6 {
        return [0.0, 0.0, 0.0];
    }
    let yd = highlight_shoulder(y);
    let scale = yd / y;
    let mapped = to_output_space([rgb[0] * scale, rgb[1] * scale, rgb[2] * scale], cs);
    project_to_gamut(mapped, yd)
}

pub fn apply_default_rgb(rgb: [f32; 3]) -> [f32; 3] {
    apply_default_rgb_cs(rgb, OutputColorSpace::SRgb)
}

pub fn apply_default_rgb_cs(rgb: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    let mapped = tone_map_luma_and_project_cs(rgb, cs);
    [
        display_encode(mapped[0]),
        display_encode(mapped[1]),
        display_encode(mapped[2]),
    ]
}

pub fn apply_rgb(rgb: [f32; 3]) -> [f32; 3] {
    apply_default_rgb(rgb)
}

pub fn apply_rgb_dcp(rgb: [f32; 3], dcp_active: bool) -> [f32; 3] {
    apply_rgb_dcp_cs(rgb, dcp_active, OutputColorSpace::SRgb)
}

pub fn apply_rgb_dcp_cs(rgb: [f32; 3], dcp_active: bool, cs: OutputColorSpace) -> [f32; 3] {
    if !dcp_active {
        return apply_default_rgb_cs(rgb, cs);
    }
    let neutral = luma(rgb).clamp(0.0, 1.0);
    let mapped = project_to_gamut(to_output_space(rgb, cs), neutral);
    [
        srgb_oetf(mapped[0].clamp(0.0, 1.0)),
        srgb_oetf(mapped[1].clamp(0.0, 1.0)),
        srgb_oetf(mapped[2].clamp(0.0, 1.0)),
    ]
}

pub fn apply_display_luma(rgb: [f32; 3]) -> f32 {
    let display = apply_default_rgb(rgb);
    0.2126 * display[0] + 0.7152 * display[1] + 0.0722 * display[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_endpoints() {
        if default_scalar(0.0).abs() > 1e-6 {
            panic!("default(0) should be 0");
        }
        let ceil = default_scalar(shared::RAW_LINEAR_CEILING);
        if !(0.98..=1.0).contains(&ceil) {
            panic!("default(ceiling) should be ~1, got {ceil}");
        }
        let mid = default_scalar(0.5);
        let expected = display_encode(0.5);
        if (mid - expected).abs() > 1e-6 {
            panic!("midtones below knee should skip the shoulder, got {mid} vs {expected}");
        }
    }

    #[test]
    fn default_monotonic_sample() {
        let mut prev = -1.0f32;
        for i in 0..=128 {
            let x = i as f32 / 128.0 * shared::RAW_LINEAR_CEILING;
            let y = default_scalar(x);
            if y < prev - 1e-6 {
                panic!("non-monotone at {x}: {y} < {prev}");
            }
            prev = y;
        }
    }

    #[test]
    fn default_rgb_neutral_matches_scalar() {
        for v in [0.0f32, 0.25, 0.5, 1.0, 2.0, shared::RAW_LINEAR_CEILING] {
            let rgb = apply_default_rgb([v, v, v]);
            let s = default_scalar(v);
            for c in rgb {
                if (c - s).abs() > 1e-5 {
                    panic!("neutral rgb must match scalar at {v}: {c} vs {s}");
                }
            }
        }
    }

    #[test]
    fn default_rgb_saturated_blue_stays_blue() {
        let out = apply_default_rgb([0.1, 0.2, 1.5]);
        if !(out[2] > out[1] && out[1] >= out[0]) {
            panic!("hue ordering R<=G<B must be preserved, got {out:?}");
        }
        if out[2] < 0.95 {
            panic!("dominant blue should reach near display white, got {out:?}");
        }
        if out[0] > 0.7 || out[1] > 0.7 {
            panic!("blue sky must not desaturate to white (R,G stay low), got {out:?}");
        }
    }

    #[test]
    fn default_rgb_projects_negative_channel() {
        let mapped = tone_map_luma_and_project_cs([-0.2, 0.5, 0.9], OutputColorSpace::SRgb);
        let mn = mapped[0].min(mapped[1]).min(mapped[2]);
        if mn < -1e-6 {
            panic!("negative channel must be projected to >= 0, got {mapped:?}");
        }
    }

    #[test]
    fn default_rgb_preserves_luminance_before_encode() {
        for rgb in [
            [0.1, 0.2, 1.5],
            [-0.2, 0.5, 0.9],
            [2.0, 1.0, 0.3],
            [0.8, 0.8, 0.8],
        ] {
            let y_in = luma(rgb);
            if y_in <= 1e-6 {
                continue;
            }
            let yd = highlight_shoulder(y_in);
            let mapped = tone_map_luma_and_project_cs(rgb, OutputColorSpace::SRgb);
            let y_out = luma(mapped);
            if (y_out - yd).abs() > 1e-4 {
                panic!(
                    "gamut projection must preserve mapped luminance {yd}, got {y_out} for {rgb:?}"
                );
            }
        }
    }

    #[test]
    fn default_rgb_bounded_and_finite() {
        let samples = [-0.5f32, -0.1, 0.0, 0.3, 0.9, 1.5, 4.0];
        for &r in &samples {
            for &g in &samples {
                for &b in &samples {
                    let out = apply_default_rgb([r, g, b]);
                    for c in out {
                        if !c.is_finite() || !(0.0..=1.0).contains(&c) {
                            panic!(
                                "output must be finite in [0,1] for {:?}: {out:?}",
                                [r, g, b]
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn p3_neutral_matches_srgb() {
        for v in [0.0f32, 0.25, 0.5, 1.0, 2.0, shared::RAW_LINEAR_CEILING] {
            let srgb = apply_default_rgb_cs([v, v, v], OutputColorSpace::SRgb);
            let p3 = apply_default_rgb_cs([v, v, v], OutputColorSpace::DisplayP3);
            for (a, b) in srgb.iter().zip(p3.iter()) {
                if (a - b).abs() > 1e-5 {
                    panic!("neutral gray must be color-space invariant at {v}: {srgb:?} vs {p3:?}");
                }
            }
        }
    }

    #[test]
    fn p3_srgb_path_is_identity() {
        let rgb = [0.6, 0.2, 0.9];
        let base = apply_default_rgb(rgb);
        let cs = apply_default_rgb_cs(rgb, OutputColorSpace::SRgb);
        if base != cs {
            panic!("sRGB color space must be byte-identical to default path: {base:?} vs {cs:?}");
        }
    }

    #[test]
    fn p3_saturated_red_less_saturated_than_srgb() {
        let rgb = [1.0, 0.0, 0.0];
        let srgb = apply_default_rgb_cs(rgb, OutputColorSpace::SRgb);
        let p3 = apply_default_rgb_cs(rgb, OutputColorSpace::DisplayP3);
        if p3 == srgb {
            panic!("P3 primary matrix must change saturated red output");
        }
        if !(p3[0] < srgb[0] && p3[1] > srgb[1] && p3[2] > srgb[2]) {
            panic!("sRGB red inside P3 should pull R down and G/B up, got {p3:?} vs {srgb:?}");
        }
    }
}
