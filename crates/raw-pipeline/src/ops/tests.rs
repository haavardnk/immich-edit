use super::LinearImage;
use super::*;
use crate::edits::{
    BasicEdits, ColorEdits, ColorGradeEdits, ColorGradeRegion, DetailEdits, Edits, GeometryEdits,
    HslBand, HslEdits, ToneEdits,
};

fn solid_image(w: usize, h: usize, rgb: [f32; 3]) -> LinearImage {
    let mut buf = Vec::with_capacity(w * h * 3);
    for _ in 0..w * h {
        buf.extend_from_slice(&rgb);
    }
    LinearImage::new(buf, w, h)
}

fn ctx() -> OpContext {
    OpContext {
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        cam_to_srgb: crate::color::identity_3x3(),
        is_raw: false,
        preview_mode: crate::frame::PreviewMode::None,
    }
}

#[test]
fn exposure_doubles_at_one_ev() {
    let mut img = solid_image(2, 2, [0.1, 0.1, 0.1]);
    let edits = Edits {
        basic: BasicEdits {
            exposure_ev: 1.0,
            ..Default::default()
        },
        ..Default::default()
    };
    exposure::ExposureOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 0.2).abs() < 1e-5);
}

#[test]
fn exposure_inactive_when_zero() {
    assert!(!exposure::ExposureOp.is_active(&Edits::default()));
}

#[test]
fn contrast_perceptual_pivot_invariant() {
    let pivot = 0.5_f32.powf(2.2);
    let mut img = solid_image(1, 1, [pivot, pivot, pivot]);
    let edits = Edits {
        basic: BasicEdits {
            contrast: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    contrast::ContrastOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - pivot).abs() < 1e-3);
}

#[test]
fn contrast_preserves_hdr_above_1() {
    let mut img = solid_image(1, 1, [1.5, 1.5, 1.5]);
    let edits = Edits {
        basic: BasicEdits {
            contrast: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    contrast::ContrastOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 1.5).abs() < 1e-5);
}

#[test]
fn contrast_negative_flattens() {
    let mut low = solid_image(1, 1, [0.2, 0.2, 0.2]);
    let mut high = solid_image(1, 1, [0.8, 0.8, 0.8]);
    let edits = Edits {
        basic: BasicEdits {
            contrast: -50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    contrast::ContrastOp
        .apply_cpu(&mut low, &ctx(), &edits)
        .unwrap();
    contrast::ContrastOp
        .apply_cpu(&mut high, &ctx(), &edits)
        .unwrap();
    let spread_after = high.rgb[0] - low.rgb[0];
    assert!(spread_after < 0.6);
}

#[test]
fn saturation_full_desaturate_yields_luma() {
    let mut img = solid_image(1, 1, [1.0, 0.0, 0.0]);
    let edits = Edits {
        basic: BasicEdits {
            saturation: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    saturation::SaturationOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let luma = 0.2126_f32;
    assert!((img.rgb[0] - luma).abs() < 1e-5);
    assert!((img.rgb[1] - luma).abs() < 1e-5);
    assert!((img.rgb[2] - luma).abs() < 1e-5);
}

#[test]
fn vibrance_boosts_low_sat_more_than_high() {
    let mut low = solid_image(1, 1, [0.55, 0.50, 0.45]);
    let mut high = solid_image(1, 1, [0.95, 0.20, 0.10]);
    let edits = Edits {
        basic: BasicEdits {
            vibrance: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let low_before = low.rgb[0] - low.rgb[2];
    let high_before = high.rgb[0] - high.rgb[2];
    vibrance::VibranceOp
        .apply_cpu(&mut low, &ctx(), &edits)
        .unwrap();
    vibrance::VibranceOp
        .apply_cpu(&mut high, &ctx(), &edits)
        .unwrap();
    let low_after = low.rgb[0] - low.rgb[2];
    let high_after = high.rgb[0] - high.rgb[2];
    let low_gain = low_after / low_before;
    let high_gain = high_after / high_before;
    assert!(low_gain > high_gain);
}

#[test]
fn highlights_lift_bright_pixels() {
    let mut img = solid_image(1, 1, [0.8, 0.8, 0.8]);
    let edits = Edits {
        tone: ToneEdits {
            highlights: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > 0.8);
}

#[test]
fn shadows_lift_dark_pixels() {
    let mut img = solid_image(1, 1, [0.2, 0.2, 0.2]);
    let edits = Edits {
        tone: ToneEdits {
            shadows: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > 0.2);
}

#[test]
fn blacks_lift_very_dark_pixels() {
    let mut img = solid_image(1, 1, [0.01, 0.01, 0.01]);
    let edits = Edits {
        tone: ToneEdits {
            blacks: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > 0.01);
    assert!(img.rgb[0] < 0.05);
}

#[test]
fn blacks_dont_affect_midtones() {
    let mut img = solid_image(1, 1, [0.3, 0.3, 0.3]);
    let edits = Edits {
        tone: ToneEdits {
            blacks: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 0.3).abs() < 1e-4);
}

#[test]
fn blacks_negative_crushes() {
    let mut img = solid_image(1, 1, [0.02, 0.02, 0.02]);
    let edits = Edits {
        tone: ToneEdits {
            blacks: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] < 0.02);
}

#[test]
fn whites_lift_very_bright_pixels() {
    let mut img = solid_image(1, 1, [0.95, 0.95, 0.95]);
    let edits = Edits {
        tone: ToneEdits {
            whites: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > 0.95);
}

#[test]
fn whites_global_gain_affects_midtones() {
    let mut img = solid_image(1, 1, [0.5, 0.5, 0.5]);
    let edits = Edits {
        tone: ToneEdits {
            whites: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 0.6667).abs() < 0.01);
}

#[test]
fn whites_negative_pulls_brights() {
    let mut img = solid_image(1, 1, [0.95, 0.95, 0.95]);
    let edits = Edits {
        tone: ToneEdits {
            whites: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] < 0.95);
}

#[test]
fn hsl_red_saturation_only_affects_red() {
    let mut red = solid_image(1, 1, [0.8, 0.2, 0.2]);
    let mut blue = solid_image(1, 1, [0.2, 0.2, 0.8]);
    let mut bands = [HslBand::default(); 8];
    bands[0] = HslBand {
        sat: -100.0,
        ..Default::default()
    };
    let edits = Edits {
        color: ColorEdits {
            hsl: HslEdits { bands },
            color_grade: Default::default(),
        },
        ..Default::default()
    };
    hsl::HslOp.apply_cpu(&mut red, &ctx(), &edits).unwrap();
    hsl::HslOp.apply_cpu(&mut blue, &ctx(), &edits).unwrap();
    let red_spread = (red.rgb[0] - red.rgb[1]).abs();
    let blue_spread = (blue.rgb[2] - blue.rgb[1]).abs();
    assert!(red_spread < 0.1);
    assert!(blue_spread > 0.5);
}

#[test]
fn white_balance_temp_warms() {
    let mut img = solid_image(1, 1, [0.4, 0.5, 0.6]);
    let edits = Edits {
        basic: BasicEdits {
            wb_temp: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    user_wb::UserWbOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > img.rgb[2]);
}

#[test]
fn user_wb_inactive_at_zero() {
    let edits = Edits::default();
    assert!(!user_wb::UserWbOp.is_active(&edits));
}

#[test]
fn user_wb_cool_shifts_blue() {
    let mut img = solid_image(1, 1, [0.4, 0.5, 0.6]);
    let edits = Edits {
        basic: BasicEdits {
            wb_temp: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    user_wb::UserWbOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[2] > img.rgb[0]);
}

#[test]
fn geometry_rotate_swaps_dims() {
    let mut img = solid_image(4, 2, [0.5, 0.5, 0.5]);
    let edits = Edits {
        geometry: GeometryEdits {
            rotate: 90,
            ..Default::default()
        },
        ..Default::default()
    };
    geometry::GeometryOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert_eq!(img.width, 2);
    assert_eq!(img.height, 4);
}

#[test]
fn registry_orders_by_stage() {
    let reg = default_registry();
    let stages: Vec<Stage> = reg.ops().iter().map(|o| o.stage()).collect();
    let mut sorted = stages.clone();
    sorted.sort();
    assert_eq!(stages, sorted);
}

#[test]
fn registry_skips_inactive_ops() {
    let reg = default_registry();
    let edits = Edits::default();
    let active: Vec<&str> = reg.active(&edits).map(|o| o.id()).collect();
    assert_eq!(active, vec!["camera_wb", "color_matrix"]);
}

#[test]
fn hsl_runs_before_saturation_and_vibrance() {
    let reg = default_registry();
    let ids: Vec<&str> = reg.ops().iter().map(|o| o.id()).collect();
    let hsl = ids.iter().position(|s| *s == "hsl").unwrap();
    let sat = ids.iter().position(|s| *s == "saturation").unwrap();
    let vib = ids.iter().position(|s| *s == "vibrance").unwrap();
    assert!(hsl < sat);
    assert!(hsl < vib);
}

#[test]
fn color_grade_identity_when_zero() {
    let mut img = solid_image(1, 1, [0.3, 0.4, 0.5]);
    let edits = Edits::default();
    color_grade::ColorGradeOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 0.3).abs() < 1e-5);
    assert!((img.rgb[1] - 0.4).abs() < 1e-5);
    assert!((img.rgb[2] - 0.5).abs() < 1e-5);
}

#[test]
fn color_grade_shadows_affect_dark_more_than_bright() {
    let mut dark = solid_image(1, 1, [0.1, 0.1, 0.1]);
    let mut bright = solid_image(1, 1, [0.9, 0.9, 0.9]);
    let edits = Edits {
        color: ColorEdits {
            hsl: HslEdits::default(),
            color_grade: ColorGradeEdits {
                shadows: ColorGradeRegion {
                    hue: 0.0,
                    sat: 100.0,
                    lum: 0.0,
                },
                ..Default::default()
            },
        },
        ..Default::default()
    };
    color_grade::ColorGradeOp
        .apply_cpu(&mut dark, &ctx(), &edits)
        .unwrap();
    color_grade::ColorGradeOp
        .apply_cpu(&mut bright, &ctx(), &edits)
        .unwrap();
    let dark_shift = dark.rgb[0] - 0.1;
    let bright_shift = bright.rgb[0] - 0.9;
    assert!(dark_shift > bright_shift);
}

#[test]
fn color_grade_global_lum_brightens() {
    let mut img = solid_image(1, 1, [0.4, 0.4, 0.4]);
    let edits = Edits {
        color: ColorEdits {
            hsl: HslEdits::default(),
            color_grade: ColorGradeEdits {
                global: ColorGradeRegion {
                    hue: 0.0,
                    sat: 0.0,
                    lum: 50.0,
                },
                ..Default::default()
            },
        },
        ..Default::default()
    };
    color_grade::ColorGradeOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > 0.4);
    assert!((img.rgb[0] - img.rgb[1]).abs() < 1e-5);
}

#[test]
fn texture_inactive_when_zero() {
    assert!(!texture::TextureOp.is_active(&Edits::default()));
}

#[test]
fn texture_positive_amplifies_detail() {
    let w: usize = 256;
    let h: usize = 128;
    let period: usize = 8;
    let mk = || {
        let mut buf = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let stripe = (((x / (period / 2)) % 2) as f32) * 0.1 + 0.45;
                let i = (y * w + x) * 3;
                buf[i] = stripe;
                buf[i + 1] = stripe;
                buf[i + 2] = stripe;
            }
        }
        LinearImage::new(buf, w, h)
    };
    let edits_pos = Edits {
        basic: BasicEdits {
            texture: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let edits_neg = Edits {
        basic: BasicEdits {
            texture: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let probe_hi = (h / 2 * w + period / 2) * 3;
    let probe_lo = (h / 2 * w + period) * 3;
    let base = mk();
    let base_spread = (base.rgb[probe_hi] - base.rgb[probe_lo]).abs();
    let mut pos = mk();
    texture::TextureOp
        .apply_cpu(&mut pos, &ctx(), &edits_pos)
        .unwrap();
    let pos_spread = (pos.rgb[probe_hi] - pos.rgb[probe_lo]).abs();
    let mut neg = mk();
    texture::TextureOp
        .apply_cpu(&mut neg, &ctx(), &edits_neg)
        .unwrap();
    let neg_spread = (neg.rgb[probe_hi] - neg.rgb[probe_lo]).abs();
    assert!(
        pos_spread > base_spread * 1.2,
        "texture +100 should amplify mid-frequency: base={base_spread} pos={pos_spread}"
    );
    assert!(
        neg_spread < base_spread * 0.9,
        "texture -100 should reduce mid-frequency: base={base_spread} neg={neg_spread}"
    );
}

#[test]
fn clarity_inactive_when_zero() {
    assert!(!clarity::ClarityOp.is_active(&Edits::default()));
}

#[test]
fn clarity_amplifies_midtones_more_than_extremes() {
    let mk = |left: f32, right: f32| {
        let mut buf = vec![0.0f32; 200 * 10 * 3];
        for y in 0..10 {
            for x in 0..200 {
                let v = if x < 100 { left } else { right };
                let i = (y * 200 + x) * 3;
                buf[i] = v;
                buf[i + 1] = v;
                buf[i + 2] = v;
            }
        }
        LinearImage::new(buf, 200, 10)
    };
    let edits = Edits {
        basic: BasicEdits {
            clarity: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut mid = mk(0.45, 0.55);
    let mut bright = mk(0.92, 1.00);
    let probe = (5 * 200 + 99) * 3;
    let mid_before = mid.rgb[probe];
    let bright_before = bright.rgb[probe];
    clarity::ClarityOp
        .apply_cpu(&mut mid, &ctx(), &edits)
        .unwrap();
    clarity::ClarityOp
        .apply_cpu(&mut bright, &ctx(), &edits)
        .unwrap();
    let mid_delta = (mid.rgb[probe] - mid_before).abs();
    let bright_delta = (bright.rgb[probe] - bright_before).abs();
    assert!(mid_delta > bright_delta);
}

#[test]
fn dehaze_inactive_when_zero() {
    assert!(!dehaze::DehazeOp.is_active(&Edits::default()));
}

#[test]
fn dehaze_positive_amplifies_edge_contrast() {
    let mk = || {
        let mut buf = vec![0.0f32; 800 * 100 * 3];
        for y in 0..100 {
            for x in 0..800 {
                let v = if x < 400 { 0.4 } else { 0.6 };
                let i = (y * 800 + x) * 3;
                buf[i] = v;
                buf[i + 1] = v;
                buf[i + 2] = v;
            }
        }
        LinearImage::new(buf, 800, 100)
    };
    let edits = Edits {
        basic: BasicEdits {
            dehaze: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut img = mk();
    let dark_probe = (50 * 800 + 399) * 3;
    let bright_probe = (50 * 800 + 400) * 3;
    dehaze::DehazeOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[dark_probe] < 0.4);
    assert!(img.rgb[bright_probe] > 0.6);
}

#[test]
fn sharpen_inactive_when_amount_zero() {
    assert!(!sharpen::SharpenOp.is_active(&Edits::default()));
}

#[test]
fn sharpen_amplifies_edge_contrast() {
    let w: usize = 64;
    let h: usize = 16;
    let mk = || {
        let mut buf = vec![0.0f32; w * h * 3];
        for y in 0..h {
            for x in 0..w {
                let v = if x < w / 2 { 0.3 } else { 0.7 };
                let i = (y * w + x) * 3;
                buf[i] = v;
                buf[i + 1] = v;
                buf[i + 2] = v;
            }
        }
        LinearImage::new(buf, w, h)
    };
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: 100.0,
            sharpen_radius: 1.0,
            sharpen_detail: 100.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let base = mk();
    let base_left = base.rgb[(8 * w + (w / 2 - 1)) * 3];
    let base_right = base.rgb[(8 * w + (w / 2)) * 3];
    let mut img = mk();
    sharpen::SharpenOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let left = img.rgb[(8 * w + (w / 2 - 1)) * 3];
    let right = img.rgb[(8 * w + (w / 2)) * 3];
    assert!(
        left < base_left && right > base_right,
        "expected sharper edge: base=({base_left},{base_right}) got=({left},{right})"
    );
}

#[test]
fn sharpen_masking_suppresses_flat_areas() {
    let w: usize = 64;
    let h: usize = 32;
    let mut buf = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let n = ((x * 7 + y * 13) % 11) as f32 / 1000.0;
            let v = 0.5 + n;
            let i = (y * w + x) * 3;
            buf[i] = v;
            buf[i + 1] = v;
            buf[i + 2] = v;
        }
    }
    let img_no_mask = LinearImage::new(buf.clone(), w, h);
    let img_mask = LinearImage::new(buf.clone(), w, h);
    let mk_edits = |masking: f64| Edits {
        detail: DetailEdits {
            sharpen_amount: 150.0,
            sharpen_radius: 1.0,
            sharpen_detail: 100.0,
            sharpen_masking: masking,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut a = img_no_mask;
    let mut b = img_mask;
    sharpen::SharpenOp
        .apply_cpu(&mut a, &ctx(), &mk_edits(0.0))
        .unwrap();
    sharpen::SharpenOp
        .apply_cpu(&mut b, &ctx(), &mk_edits(100.0))
        .unwrap();
    let diff_no_mask: f32 = a
        .rgb
        .iter()
        .zip(buf.iter())
        .map(|(x, y)| (x - y).abs())
        .sum();
    let diff_mask: f32 = b
        .rgb
        .iter()
        .zip(buf.iter())
        .map(|(x, y)| (x - y).abs())
        .sum();
    assert!(
        diff_mask < diff_no_mask * 0.25,
        "masking should suppress noise sharpening: no_mask={diff_no_mask} masked={diff_mask}"
    );
}
