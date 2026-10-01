use super::*;

#[test]
fn default_color_curve_shapes_midtones() {
    let profile = ResolvedDcp::default_color();
    if profile.base_table.is_some() || profile.look_table.is_some() {
        panic!("default color must not carry hue/sat tables");
    }
    let curve = profile.tone_curve.as_deref();
    let at = |v: f32| {
        crate::color::apply_dcp_finish(None, curve, &profile.to_pp, &profile.from_pp, [v, v, v])[1]
    };
    let shadow = at(0.1);
    if shadow >= 0.1 {
        panic!("default color must deepen shadows, got {shadow} at 0.1");
    }
    let midtone = at(0.5);
    if midtone <= 0.5 {
        panic!("default color must lift midtones, got {midtone} at 0.5");
    }
}
fn saturation_vibrance_pixel(rgb: [f32; 3], saturation: f64, vibrance: f64) -> [f32; 3] {
    let mut img = solid_image(1, 1, rgb);
    let edits = Edits {
        basic: BasicEdits {
            saturation,
            vibrance,
            ..Default::default()
        },
        ..Default::default()
    };
    saturation::SaturationOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    vibrance::VibranceOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    [img.rgb[0], img.rgb[1], img.rgb[2]]
}

#[test]
fn saturation_full_desaturate_keeps_oklab_lightness() {
    let rgb = [1.0, 0.0, 0.0];
    let out = saturation_vibrance_pixel(rgb, -100.0, 0.0);
    let gray = crate::math::linear_srgb_to_oklab(rgb)[0].powi(3);
    if out.iter().any(|v| (v - gray).abs() > 1e-4) {
        panic!("desaturated red gave {out:?}, want {gray}");
    }
}

#[test]
fn saturation_and_vibrance_keep_oklab_hue_and_lightness() {
    let hue = |lab: [f32; 3]| lab[2].atan2(lab[1]).to_degrees();
    let chroma = |lab: [f32; 3]| lab[1].hypot(lab[2]);
    for (saturation, vibrance) in [(50.0, 0.0), (-50.0, 0.0), (0.0, 80.0), (0.0, -60.0)] {
        for rgb in [
            [0.05, 0.1, 0.6],
            [0.6, 0.08, 0.05],
            [0.1, 0.45, 0.08],
            [0.5, 0.3, 0.2],
            [0.02, 0.015, 0.01],
        ] {
            let before = crate::math::linear_srgb_to_oklab(rgb);
            let after = crate::math::linear_srgb_to_oklab(saturation_vibrance_pixel(
                rgb, saturation, vibrance,
            ));
            let dh = crate::math::hue_dist(hue(after), hue(before));
            if (after[0] - before[0]).abs() > 1e-4 || dh > 0.1 {
                panic!("sat {saturation} vib {vibrance} {rgb:?}: lab {before:?} -> {after:?}");
            }
            let ratio = chroma(after) / chroma(before);
            if vibrance == 0.0 && (ratio - (1.0 + saturation as f32 / 100.0)).abs() > 1e-3 {
                panic!("saturation {saturation} on {rgb:?} scaled chroma by {ratio}");
            }
        }
    }
}

#[test]
fn vibrance_ignores_exposure() {
    for rgb in [[0.3, 0.2, 0.12], [0.08, 0.12, 0.3]] {
        let base = saturation_vibrance_pixel(rgb, 0.0, 70.0);
        for k in [0.125_f32, 4.0] {
            let scaled = saturation_vibrance_pixel(rgb.map(|v| v * k), 0.0, 70.0);
            if scaled
                .iter()
                .zip(base)
                .any(|(s, b)| (s / k - b).abs() > 1e-4)
            {
                panic!("{rgb:?} x{k}: {scaled:?} vs {base:?}");
            }
        }
    }
}

fn bw_edits(bw: crate::edits::BwEdits) -> Edits {
    Edits {
        color: ColorEdits {
            bw: crate::edits::BwEdits {
                enabled: true,
                ..bw
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn bw_pixel(rgb: [f32; 3], bw: crate::edits::BwEdits) -> [f32; 3] {
    let mut img = solid_image(1, 1, rgb);
    bw::BwOp.apply_cpu(&mut img, &ctx(), &bw_edits(bw)).unwrap();
    [img.rgb[0], img.rgb[1], img.rgb[2]]
}

#[test]
fn black_and_white_without_a_mix_is_luminance() {
    let rgb = [0.8, 0.3, 0.1];
    let out = bw_pixel(rgb, Default::default());
    let luma = crate::math::luma(rgb[0], rgb[1], rgb[2]);
    if out.iter().any(|v| (v - luma).abs() > 1e-5) {
        panic!("neutral black and white gave {out:?}, want {luma}");
    }
}

#[test]
fn black_and_white_mix_moves_only_its_colour() {
    let mix = |red: f64| crate::edits::BwEdits {
        mix: crate::edits::BwMix {
            red,
            ..Default::default()
        },
        ..Default::default()
    };
    for (rgb, moves) in [
        ([0.7, 0.05, 0.05], true),
        ([0.05, 0.05, 0.7], false),
        ([0.4, 0.4, 0.4], false),
    ] {
        let [bright, _, _] = bw_pixel(rgb, mix(100.0));
        let [plain, _, _] = bw_pixel(rgb, mix(0.0));
        let [dark, _, _] = bw_pixel(rgb, mix(-100.0));
        if moves != (bright > plain * 1.5 && dark < plain / 1.5) {
            panic!("{rgb:?}: {dark} {plain} {bright}");
        }
    }
}

#[test]
fn black_and_white_tint_keeps_luminance_and_follows_balance() {
    let tint = crate::edits::BwEdits {
        shadows: crate::edits::BwTint {
            hue: 220.0,
            sat: 100.0,
        },
        highlights: crate::edits::BwTint {
            hue: 40.0,
            sat: 100.0,
        },
        ..Default::default()
    };
    let dark = bw_pixel([0.02, 0.02, 0.02], tint);
    let bright = bw_pixel([0.8, 0.8, 0.8], tint);
    if dark[2] <= dark[0] || bright[0] <= bright[2] {
        panic!("shadows {dark:?} should be blue, highlights {bright:?} warm");
    }
    for (out, gray) in [(dark, 0.02), (bright, 0.8)] {
        let luma = crate::math::luma(out[0], out[1], out[2]);
        if (luma - gray).abs() > 1e-4 {
            panic!("tint changed luminance: {luma} vs {gray}");
        }
    }
    let mid = [0.2, 0.2, 0.2];
    let warmth = |balance: f64| {
        let out = bw_pixel(mid, crate::edits::BwEdits { balance, ..tint });
        out[0] - out[2]
    };
    if warmth(100.0) <= warmth(-100.0) {
        panic!("a positive balance must favour the highlight tint");
    }
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
fn vibrance_skin_protected_vs_non_skin() {
    let mut skin = solid_image(1, 1, [0.7, 0.55, 0.45]);
    let mut non_skin = solid_image(1, 1, [0.7, 0.45, 0.55]);
    let edits = Edits {
        basic: BasicEdits {
            vibrance: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let skin_before = skin.rgb[0] - skin.rgb[2];
    let non_before = non_skin.rgb[0] - non_skin.rgb[2];
    vibrance::VibranceOp
        .apply_cpu(&mut skin, &ctx(), &edits)
        .unwrap();
    vibrance::VibranceOp
        .apply_cpu(&mut non_skin, &ctx(), &edits)
        .unwrap();
    let skin_gain = (skin.rgb[0] - skin.rgb[2]).abs() / skin_before.abs();
    let non_gain = (non_skin.rgb[0] - non_skin.rgb[2]).abs() / non_before.abs();
    if skin_gain >= non_gain {
        panic!("skin not damped vs non-skin: skin_gain={skin_gain} non_gain={non_gain}");
    }
}

#[test]
fn vibrance_gray_stays_gray() {
    let mut img = solid_image(1, 1, [0.5, 0.5, 0.5]);
    let edits = Edits {
        basic: BasicEdits {
            vibrance: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    vibrance::VibranceOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    for v in &img.rgb {
        if (v - 0.5).abs() > 1e-5 {
            panic!("gray shifted: {v}");
        }
    }
}

#[test]
fn vibrance_high_sat_barely_moves_on_positive() {
    let mut img = solid_image(1, 1, [0.95, 0.05, 0.05]);
    let before_spread = img.rgb[0] - img.rgb[2];
    let edits = Edits {
        basic: BasicEdits {
            vibrance: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    vibrance::VibranceOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let after_spread = img.rgb[0] - img.rgb[2];
    let delta = (after_spread - before_spread).abs();
    if delta > 0.05 {
        panic!("high-sat pixel moved too much: delta={delta}");
    }
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
            lut_3d: Default::default(),
            dcp: Default::default(),
            bw: Default::default(),
        },
        ..Default::default()
    };
    hsl::HslOp.apply_cpu(&mut red, &ctx(), &edits).unwrap();
    hsl::HslOp.apply_cpu(&mut blue, &ctx(), &edits).unwrap();
    let red_spread = (red.rgb[0] - red.rgb[1]).abs();
    let blue_spread = (blue.rgb[2] - blue.rgb[1]).abs();
    assert!(red_spread < 0.3);
    assert!(blue_spread > 0.5);
}

#[test]
fn hsl_gray_pixel_barely_shifts_under_max_bands() {
    let mut gray = solid_image(1, 1, [0.5, 0.5, 0.5]);
    let bands = [HslBand {
        hue: 100.0,
        sat: 100.0,
        lum: 100.0,
    }; 8];
    let edits = Edits {
        color: ColorEdits {
            hsl: HslEdits { bands },
            color_grade: Default::default(),
            lut_3d: Default::default(),
            dcp: Default::default(),
            bw: Default::default(),
        },
        ..Default::default()
    };
    hsl::HslOp.apply_cpu(&mut gray, &ctx(), &edits).unwrap();
    for v in &gray.rgb {
        if (v - 0.5).abs() > 1e-3 {
            panic!("gray pixel shifted: {v}");
        }
    }
}

#[test]
fn hsl_band_overlap_does_not_stack() {
    let mut p1 = solid_image(1, 1, [0.8, 0.4, 0.2]);
    let mut p2 = solid_image(1, 1, [0.8, 0.4, 0.2]);
    let mut single = [HslBand::default(); 8];
    single[1] = HslBand {
        sat: 100.0,
        ..Default::default()
    };
    let mut both = [HslBand::default(); 8];
    both[1] = HslBand {
        sat: 100.0,
        ..Default::default()
    };
    both[2] = HslBand {
        sat: 100.0,
        ..Default::default()
    };
    let e1 = Edits {
        color: ColorEdits {
            hsl: HslEdits { bands: single },
            color_grade: Default::default(),
            lut_3d: Default::default(),
            dcp: Default::default(),
            bw: Default::default(),
        },
        ..Default::default()
    };
    let e2 = Edits {
        color: ColorEdits {
            hsl: HslEdits { bands: both },
            color_grade: Default::default(),
            lut_3d: Default::default(),
            dcp: Default::default(),
            bw: Default::default(),
        },
        ..Default::default()
    };
    hsl::HslOp.apply_cpu(&mut p1, &ctx(), &e1).unwrap();
    hsl::HslOp.apply_cpu(&mut p2, &ctx(), &e2).unwrap();
    let sat_one = (p1.rgb[0] - p1.rgb[2]).abs();
    let sat_both = (p2.rgb[0] - p2.rgb[2]).abs();
    if sat_both > sat_one + 0.15 {
        panic!("overlapping bands stacked: one={sat_one} both={sat_both}");
    }
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
            lut_3d: Default::default(),
            dcp: Default::default(),
            bw: Default::default(),
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
            lut_3d: Default::default(),
            dcp: Default::default(),
            bw: Default::default(),
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
fn dcp_base_table_sees_values_before_baseline_exposure() {
    let map = std::sync::Arc::new(crate::dcp::HueSatMap {
        hue_div: 6,
        sat_div: 2,
        val_div: 4,
        encoding: crate::dcp::HsvEncoding::Linear,
        data: (0..48)
            .map(|i| {
                [
                    i as f32 - 24.0,
                    0.8 + (i % 5) as f32 * 0.1,
                    0.7 + (i % 3) as f32 * 0.2,
                ]
            })
            .collect(),
    });
    let render = |gain: f32, rgb: [f32; 3]| -> [f32; 3] {
        let mut c = ctx();
        c.render.dcp = Some(std::sync::Arc::new(ResolvedDcp {
            base_table: Some(map.clone()),
            baseline_gain: gain,
            ..ResolvedDcp::default_color()
        }));
        let mut img = solid_image(1, 1, rgb);
        dcp_profile::DcpProfileOp
            .apply_cpu(&mut img, &c, &Edits::default())
            .unwrap();
        [img.rgb[0], img.rgb[1], img.rgb[2]]
    };
    for rgb in [[0.3, 0.2, 0.1], [0.05, 0.4, 0.2], [0.45, 0.4, 0.42]] {
        let base = render(1.0, rgb);
        let lifted = render(2.0, rgb.map(|v| v * 2.0));
        for ch in 0..3 {
            if (lifted[ch] - 2.0 * base[ch]).abs() > 1e-5 {
                panic!("{rgb:?} ch{ch}: table saw lifted values {lifted:?} vs 2x{base:?}");
            }
        }
    }
}

#[test]
fn hsl_bright_pixel_changes_only_under_its_band() {
    let src = [1.4, 0.6, 0.3];
    for (band, moved) in [(5, false), (0, true)] {
        let mut img = solid_image(1, 1, src);
        let mut bands = [HslBand::default(); 8];
        bands[band] = HslBand {
            sat: -100.0,
            ..Default::default()
        };
        let edits = Edits {
            color: ColorEdits {
                hsl: HslEdits { bands },
                ..Default::default()
            },
            ..Default::default()
        };
        hsl::HslOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        let delta = img
            .rgb
            .iter()
            .zip(src)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        if (delta > 1e-3) != moved {
            panic!("band {band}: {src:?} -> {:?}", img.rgb);
        }
    }
}

fn all_hsl_bands(band: HslBand) -> Edits {
    Edits {
        color: ColorEdits {
            hsl: HslEdits { bands: [band; 8] },
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn hsl_desaturate_keeps_perceptual_lightness() {
    let src = [0.05, 0.1, 0.4];
    let mut img = solid_image(1, 1, src);
    let edits = all_hsl_bands(HslBand {
        sat: -100.0,
        ..Default::default()
    });
    hsl::HslOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
    let out = [img.rgb[0], img.rgb[1], img.rgb[2]];
    let spread = out.iter().fold(0.0f32, |m, v| m.max((v - out[0]).abs()));
    let lightness_in = crate::math::linear_srgb_to_oklab(src)[0];
    let lightness_out = crate::math::linear_srgb_to_oklab(out)[0];
    if spread > 1e-3 || (lightness_out - lightness_in).abs() > 1e-3 {
        panic!("sky desaturated to {out:?}: L {lightness_in} -> {lightness_out}");
    }
}

#[test]
fn hsl_lightness_scales_exposure_and_keeps_ratios() {
    let cases = [
        ([0.01, 0.02, 0.08], -50.0, (-0.5 * hsl::HSL_LUM_EV).exp2()),
        ([1.4, 0.6, 0.3], 100.0, hsl::HSL_LUM_EV.exp2()),
    ];
    for (src, lum, gain) in cases {
        let mut img = solid_image(1, 1, src);
        let edits = all_hsl_bands(HslBand {
            lum,
            ..Default::default()
        });
        hsl::HslOp.apply_cpu(&mut img, &ctx(), &edits).unwrap();
        let off = (0..3)
            .map(|c| (img.rgb[c] / (src[c] * gain) - 1.0).abs())
            .fold(0.0f32, f32::max);
        if off > 2e-3 {
            panic!("lum {lum}: {src:?} -> {:?}, want x{gain}", img.rgb);
        }
    }
}
