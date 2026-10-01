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
    assert!((img.rgb[0] - 1.0).abs() < 0.01);
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
