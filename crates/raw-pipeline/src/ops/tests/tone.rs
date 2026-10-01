use super::*;

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
fn highlights_recover_bright_pixels() {
    let mut img = solid_image(1, 1, [0.9, 0.9, 0.9]);
    let edits = Edits {
        tone: ToneEdits {
            highlights: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] < 0.9);
}

#[test]
fn highlights_clip_excess_at_full_negative() {
    let mut img = solid_image(1, 1, [1.5, 1.5, 1.5]);
    let edits = Edits {
        tone: ToneEdits {
            highlights: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] <= 1.0 + 1e-4);
}

#[test]
fn highlights_dont_touch_shadows() {
    let mut img = solid_image(1, 1, [0.1, 0.1, 0.1]);
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
    assert!((img.rgb[0] - 0.1).abs() < 1e-3);
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
fn flat_shadows_lift_ignores_tone_gains_before_it() {
    let render = |basic: &BasicEdits, tone: &ToneEdits| -> f32 {
        let mut img = solid_image(64, 64, [0.03, 0.03, 0.03]);
        let edits = Edits {
            basic: basic.clone(),
            tone: tone.clone(),
            ..Default::default()
        };
        crate::cpu::run_pipeline_ops(
            &mut img,
            &ctx(),
            &edits,
            &crate::mask_raster::empty_rasters(),
            None,
        )
        .unwrap();
        img.rgb[(32 * 64 + 32) * 3 + 1]
    };
    let cases = [
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (2.0, 0.0, 0.0),
        (0.0, 60.0, 0.0),
        (0.0, 0.0, 80.0),
    ];
    for (exposure_ev, brightness, whites) in cases {
        let basic = BasicEdits {
            exposure_ev,
            brightness,
            ..Default::default()
        };
        let plain = render(
            &basic,
            &ToneEdits {
                whites,
                ..Default::default()
            },
        );
        let lifted = render(
            &basic,
            &ToneEdits {
                whites,
                shadows: 50.0,
                ..Default::default()
            },
        );
        let expected = tone_regions::shadows_mult(plain, plain, 0.5);
        let got = lifted / plain;
        if (got - expected).abs() > 1e-3 {
            panic!(
                "ev {exposure_ev} brightness {brightness} whites {whites}: flat lift {got}, expected {expected}"
            );
        }
    }
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
fn whites_move_highlights_not_blacks() {
    let levels: Vec<f32> = (0..=16).map(|i| i as f32 / 16.0).collect();
    for whites in [100.0, -100.0] {
        let mut pixels: Vec<[f32; 3]> = levels.iter().map(|&v| [v, v, v]).collect();
        pixels.push([0.6, 0.3, 0.1]);
        let width = pixels.len();
        let mut img = LinearImage::new(pixels.iter().flatten().copied().collect(), width, 1);
        let edits = Edits {
            tone: ToneEdits {
                whites,
                ..Default::default()
            },
            ..Default::default()
        };
        tone_regions::ToneRegionsOp
            .apply_cpu(&mut img, &ctx(), &edits)
            .unwrap();
        let out: Vec<f32> = img.rgb.chunks(3).map(|p| p[1]).collect();
        let stops = |v: f32| {
            (out[levels.iter().position(|&l| l == v).unwrap()] / v)
                .log2()
                .abs()
        };
        let colour = &img.rgb[img.rgb.len() - 3..];
        if stops(0.125) > 0.35 || stops(1.0) < 0.95 || stops(1.0) <= stops(0.5) {
            panic!(
                "whites {whites}: shadow {} highlight {} stops",
                stops(0.125),
                stops(1.0)
            );
        }
        if out[0] != 0.0 || out.windows(2).take(levels.len() - 1).any(|w| w[1] <= w[0]) {
            panic!("whites {whites}: not monotonic from black: {out:?}");
        }
        if (colour[0] / colour[2] - 6.0).abs() > 1e-3 {
            panic!("whites {whites}: hue shifted to {colour:?}");
        }
    }
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
fn highlights_neg_desaturates_clipped_color() {
    let mut img = solid_image(1, 1, [2.0, 1.5, 1.0]);
    let edits = Edits {
        tone: ToneEdits {
            highlights: -100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    tone_regions::ToneRegionsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let spread_before = 2.0f32 - 1.0f32;
    let spread_after = img.rgb[0] - img.rgb[2];
    if spread_after > spread_before * 0.5 {
        panic!(
            "clipped specular not desaturated: r={} g={} b={}",
            img.rgb[0], img.rgb[1], img.rgb[2]
        );
    }
}
