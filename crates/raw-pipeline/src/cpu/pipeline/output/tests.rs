use super::*;
use crate::tone::{srgb_oetf, srgb_oetf_scalar};

#[test]
fn display_ready_output_skips_tone_mapping() {
    let (rgb, _, _, _) = finish_output(
        vec![0.5, 0.5, 0.5],
        1,
        1,
        false,
        true,
        None,
        None,
        OutputColorSpace::SRgb,
        false,
        false,
    );
    if rgb.iter().any(|value| !(126..=129).contains(value)) {
        panic!("expected display-ready midpoint, got {rgb:?}");
    }
}

#[test]
fn gamut_warn_paints_out_of_gamut_pixels() {
    let (rgb, _, _, _) = finish_output(
        vec![0.9, -0.1, 0.2],
        1,
        1,
        false,
        false,
        None,
        None,
        OutputColorSpace::SRgb,
        true,
        false,
    );
    if rgb != vec![255, 0, 255] {
        panic!("expected magenta gamut warning, got {rgb:?}");
    }
}

#[test]
fn gamut_warn_ignores_bright_in_gamut_pixels() {
    let (rgb, _, _, _) = finish_output(
        vec![4.0, 0.0, 0.0],
        1,
        1,
        false,
        false,
        None,
        None,
        OutputColorSpace::SRgb,
        true,
        false,
    );
    if rgb == vec![255, 0, 255] {
        panic!("bright in-gamut red must not be flagged out of gamut");
    }
}

#[test]
fn gamut_warn_leaves_in_gamut_pixels() {
    let (rgb, _, _, _) = finish_output(
        vec![0.5, 0.5, 0.5],
        1,
        1,
        false,
        false,
        None,
        None,
        OutputColorSpace::SRgb,
        true,
        false,
    );
    if rgb == vec![255, 0, 255] {
        panic!("neutral gray must not be flagged out of gamut");
    }
}

#[test]
fn clip_warn_paints_blown_and_crushed_pixels() {
    let cases = [
        ([4.0f32, 4.0, 4.0], Some(crate::warn::HIGHLIGHT_WARN_RGB)),
        ([0.05, 4.0, 0.05], Some(crate::warn::HIGHLIGHT_WARN_RGB)),
        ([0.0, 0.0, 0.0], Some(crate::warn::SHADOW_WARN_RGB)),
        ([0.5, 0.5, 0.5], None),
        ([0.2, 0.25, 0.22], None),
    ];
    for (linear, want) in cases {
        let (rgb, _, _, _) = finish_output(
            linear.to_vec(),
            1,
            1,
            false,
            false,
            None,
            None,
            OutputColorSpace::SRgb,
            false,
            true,
        );
        match want {
            Some(paint) => {
                if rgb != paint.to_vec() {
                    panic!("clip warning for {linear:?} = {rgb:?}, want {paint:?}");
                }
            }
            None => {
                if rgb == crate::warn::HIGHLIGHT_WARN_RGB.to_vec()
                    || rgb == crate::warn::SHADOW_WARN_RGB.to_vec()
                {
                    panic!("midtone {linear:?} must not be flagged, got {rgb:?}");
                }
            }
        }
    }
}

#[test]
fn clip_warn_off_leaves_blown_pixels_alone() {
    let (rgb, _, _, _) = finish_output(
        vec![4.0, 4.0, 4.0],
        1,
        1,
        false,
        false,
        None,
        None,
        OutputColorSpace::SRgb,
        false,
        false,
    );
    if rgb != vec![255, 255, 255] {
        panic!("expected untouched white, got {rgb:?}");
    }
}

#[test]
fn oetf_lut_matches_scalar_for_u8() {
    let mut x = 0.0f32;
    let mut max_err: f32 = 0.0;
    let mut max_q_diff: i32 = 0;
    while x <= 1.0 {
        let lut = srgb_oetf(x);
        let exact = srgb_oetf_scalar(x);
        let err = (lut - exact).abs();
        if err > max_err {
            max_err = err;
        }
        let q_lut = (lut.clamp(0.0, 1.0) * 255.0).round() as i32;
        let q_exact = (exact.clamp(0.0, 1.0) * 255.0).round() as i32;
        let d = (q_lut - q_exact).abs();
        if d > max_q_diff {
            max_q_diff = d;
        }
        x += 1.0 / 8192.0;
    }
    if max_q_diff > 1 {
        panic!("u8 quantization differs by {max_q_diff} (max abs err {max_err})");
    }
}

#[test]
fn oetf_lut_matches_scalar_for_u16() {
    let mut x = 0.0f32;
    let mut max_diff: i32 = 0;
    while x <= 1.0 {
        let lut = srgb_oetf(x);
        let exact = srgb_oetf_scalar(x);
        let q_lut = (lut.clamp(0.0, 1.0) * 65535.0).round() as i32;
        let q_exact = (exact.clamp(0.0, 1.0) * 65535.0).round() as i32;
        let d = (q_lut - q_exact).abs();
        if d > max_diff {
            max_diff = d;
        }
        x += 1.0 / 16384.0;
    }
    if max_diff > 16 {
        panic!("u16 quantization differs by {max_diff}");
    }
}

#[test]
fn dither_is_deterministic_per_pixel() {
    let a = quantize_u8_dithered(0.5, 12, 7, 1);
    let b = quantize_u8_dithered(0.5, 12, 7, 1);
    if a != b {
        panic!("dither must be deterministic for identical coords, got {a} vs {b}");
    }
}

#[test]
fn dither_stays_within_one_lsb() {
    let v = 100.0 / 255.0;
    let base = (v * 255.0f32).round() as i32;
    let mut x = 0u32;
    while x < 64 {
        let q = quantize_u8_dithered(v, x, 0, 0) as i32;
        if (q - base).abs() > 1 {
            panic!("dither perturbation exceeds 1 LSB at x={x}: {q} vs {base}");
        }
        x += 1;
    }
}

#[test]
fn dither_breaks_bands_on_subtle_gradient() {
    let lo = 100.4 / 255.0;
    let hi = 100.6 / 255.0;
    let mut seen_lo = false;
    let mut seen_hi = false;
    let mut x = 0u32;
    while x < 256 {
        let v = lo + (hi - lo) * (x as f32 / 255.0);
        let q = quantize_u8_dithered(v, x, 3, 2);
        if q == 100 {
            seen_lo = true;
        }
        if q == 101 {
            seen_hi = true;
        }
        x += 1;
    }
    if !(seen_lo && seen_hi) {
        panic!("dither should distribute a sub-LSB gradient across both bins");
    }
}
