pub mod shared;
pub mod wgsl;

use crate::frame::OutputColorSpace;
use crate::math::luma;
use shared::{
    OETF_LUT_SIZE, SRGB_OETF_GAMMA, SRGB_OETF_GAMMA_OFFSET, SRGB_OETF_GAMMA_SCALE,
    SRGB_OETF_LINEAR_CUTOFF, SRGB_OETF_LINEAR_SLOPE,
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
    let neutral = luma(rgb[0], rgb[1], rgb[2]).clamp(0.0, 1.0);
    project_to_gamut(to_output_space(rgb, cs), neutral)
}

pub fn apply_rgb(rgb: [f32; 3]) -> [f32; 3] {
    apply_rgb_cs(rgb, OutputColorSpace::SRgb)
}

pub fn apply_rgb_cs(rgb: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    let mapped = tone_map_luma_and_project_cs(rgb, cs);
    [
        srgb_oetf(mapped[0].clamp(0.0, 1.0)),
        srgb_oetf(mapped[1].clamp(0.0, 1.0)),
        srgb_oetf(mapped[2].clamp(0.0, 1.0)),
    ]
}

pub fn apply_display_luma(rgb: [f32; 3]) -> f32 {
    let display = apply_rgb(rgb);
    luma(display[0], display[1], display[2])
}

fn below_gamut(c: [f32; 3]) -> bool {
    c.iter().any(|&v| v < -1e-4)
}

pub fn is_out_of_gamut(rgb: [f32; 3], cs: OutputColorSpace) -> bool {
    below_gamut(to_output_space(rgb, cs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_endpoints() {
        let black = apply_rgb([0.0, 0.0, 0.0]);
        if black.iter().any(|c| c.abs() > 1e-6) {
            panic!("black must stay black, got {black:?}");
        }
        let white = apply_rgb([1.0, 1.0, 1.0]);
        if white.iter().any(|c| (c - 1.0).abs() > 1e-5) {
            panic!("linear white must map to display white, got {white:?}");
        }
        let mid = apply_rgb([0.5, 0.5, 0.5]);
        let expected = srgb_oetf(0.5);
        if (mid[0] - expected).abs() > 1e-6 {
            panic!(
                "flat path must be a bare OETF, got {} vs {expected}",
                mid[0]
            );
        }
    }

    #[test]
    fn flat_monotonic_sample() {
        let mut prev = -1.0f32;
        for i in 0..=128 {
            let x = i as f32 / 128.0 * shared::RAW_LINEAR_CEILING;
            let y = apply_rgb([x, x, x])[0];
            if y < prev - 1e-6 {
                panic!("non-monotone at {x}: {y} < {prev}");
            }
            prev = y;
        }
    }

    #[test]
    fn flat_neutral_matches_oetf() {
        for v in [0.0f32, 0.25, 0.5, 1.0, 2.0, shared::RAW_LINEAR_CEILING] {
            let rgb = apply_rgb([v, v, v]);
            let s = srgb_oetf(v.clamp(0.0, 1.0));
            for c in rgb {
                if (c - s).abs() > 1e-5 {
                    panic!("neutral rgb must match scalar oetf at {v}: {c} vs {s}");
                }
            }
        }
    }

    #[test]
    fn flat_saturated_blue_stays_blue() {
        let out = apply_rgb([0.1, 0.2, 1.5]);
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
    fn flat_projects_negative_channel() {
        let mapped = tone_map_luma_and_project_cs([-0.2, 0.5, 0.9], OutputColorSpace::SRgb);
        let mn = mapped[0].min(mapped[1]).min(mapped[2]);
        if mn < -1e-6 {
            panic!("negative channel must be projected to >= 0, got {mapped:?}");
        }
    }

    #[test]
    fn flat_rolls_highlights_toward_neutral() {
        let mapped = tone_map_luma_and_project_cs([1.5, 0.4, 0.2], OutputColorSpace::SRgb);
        let mx = mapped[0].max(mapped[1]).max(mapped[2]);
        if mx > 1.0 + 1e-4 {
            panic!("above-white must be projected to <= 1, got {mapped:?}");
        }
        if !(mapped[0] > mapped[1] && mapped[1] > mapped[2]) {
            panic!("channel ordering must survive the projection, got {mapped:?}");
        }
        let white = tone_map_luma_and_project_cs([2.0, 1.0, 0.3], OutputColorSpace::SRgb);
        if white.iter().any(|c| (c - 1.0).abs() > 1e-4) {
            panic!("above-white luma must land on display white, got {white:?}");
        }
    }

    #[test]
    fn flat_bounded_and_finite() {
        let samples = [-0.5f32, -0.1, 0.0, 0.3, 0.9, 1.5, 4.0];
        for &r in &samples {
            for &g in &samples {
                for &b in &samples {
                    let out = apply_rgb([r, g, b]);
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
            let srgb = apply_rgb_cs([v, v, v], OutputColorSpace::SRgb);
            let p3 = apply_rgb_cs([v, v, v], OutputColorSpace::DisplayP3);
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
        let base = apply_rgb(rgb);
        let cs = apply_rgb_cs(rgb, OutputColorSpace::SRgb);
        if base != cs {
            panic!("sRGB color space must be byte-identical to default path: {base:?} vs {cs:?}");
        }
    }

    #[test]
    fn p3_saturated_red_less_saturated_than_srgb() {
        let rgb = [1.0, 0.0, 0.0];
        let srgb = apply_rgb_cs(rgb, OutputColorSpace::SRgb);
        let p3 = apply_rgb_cs(rgb, OutputColorSpace::DisplayP3);
        if p3 == srgb {
            panic!("P3 primary matrix must change saturated red output");
        }
        if !(p3[0] < srgb[0] && p3[1] > srgb[1] && p3[2] > srgb[2]) {
            panic!("sRGB red inside P3 should pull R down and G/B up, got {p3:?} vs {srgb:?}");
        }
    }
}
