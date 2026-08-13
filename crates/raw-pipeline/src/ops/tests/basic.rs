use super::*;

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
fn brightness_zero_is_identity() {
    let mut img = solid_image(2, 2, [0.3, 0.6, 0.2]);
    let edits = Edits::default();
    assert!(!brightness::BrightnessOp.is_active(&edits));
    brightness::BrightnessOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 0.3).abs() < 1e-6);
    assert!((img.rgb[1] - 0.6).abs() < 1e-6);
    assert!((img.rgb[2] - 0.2).abs() < 1e-6);
}

#[test]
fn brightness_lifts_midtone_preserves_endpoints() {
    let mut mid = solid_image(1, 1, [0.18, 0.18, 0.18]);
    let mut black = solid_image(1, 1, [0.0, 0.0, 0.0]);
    let mut white = solid_image(1, 1, [1.0, 1.0, 1.0]);
    let edits = Edits {
        basic: BasicEdits {
            brightness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    brightness::BrightnessOp
        .apply_cpu(&mut mid, &ctx(), &edits)
        .unwrap();
    brightness::BrightnessOp
        .apply_cpu(&mut black, &ctx(), &edits)
        .unwrap();
    brightness::BrightnessOp
        .apply_cpu(&mut white, &ctx(), &edits)
        .unwrap();
    assert!(mid.rgb[0] > 0.22);
    assert!(black.rgb[0].abs() < 1e-6);
    assert!((white.rgb[0] - 1.0).abs() < 1e-3);
}

#[test]
fn brightness_negative_compresses_midtone() {
    let mut img = solid_image(1, 1, [0.5, 0.5, 0.5]);
    let edits = Edits {
        basic: BasicEdits {
            brightness: -50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    brightness::BrightnessOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] < 0.5);
    assert!(img.rgb[0] > 0.0);
}

#[test]
fn brightness_preserves_channel_ratios() {
    let mut img = solid_image(1, 1, [0.4, 0.2, 0.1]);
    let edits = Edits {
        basic: BasicEdits {
            brightness: 30.0,
            ..Default::default()
        },
        ..Default::default()
    };
    brightness::BrightnessOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    let ratio_rg_in = 0.4 / 0.2;
    let ratio_rg_out = img.rgb[0] / img.rgb[1];
    assert!((ratio_rg_in - ratio_rg_out).abs() < 1e-4);
}

#[test]
fn brightness_protects_clipped_color_channel() {
    let mut img = solid_image(1, 1, [2.0, 0.05, 0.05]);
    let edits = Edits {
        basic: BasicEdits {
            brightness: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    brightness::BrightnessOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!((img.rgb[0] - 2.0).abs() < 0.1);
}

#[test]
fn brightness_fused_matches_unfused() {
    let src: [f32; 12] = [0.3, 0.4, 0.2, 0.1, 0.5, 0.7, 0.9, 0.3, 0.6, 0.2, 0.8, 0.4];
    let mut a = LinearImage::new(src.to_vec(), 2, 2);
    let mut b = LinearImage::new(src.to_vec(), 2, 2);
    let edits = Edits {
        basic: BasicEdits {
            brightness: 40.0,
            ..Default::default()
        },
        ..Default::default()
    };
    brightness::BrightnessOp
        .apply_cpu(&mut a, &ctx(), &edits)
        .unwrap();
    let fused = brightness::BrightnessOp.cpu_fused(&edits, &ctx()).unwrap();
    for i in 0..b.rgb.len() / 3 {
        let mut r = b.rgb[i * 3];
        let mut g = b.rgb[i * 3 + 1];
        let mut bb = b.rgb[i * 3 + 2];
        crate::cpu::fused::apply_one(&fused, i, &mut r, &mut g, &mut bb);
        b.rgb[i * 3] = r;
        b.rgb[i * 3 + 1] = g;
        b.rgb[i * 3 + 2] = bb;
    }
    for (x, y) in a.rgb.iter().zip(b.rgb.iter()) {
        assert!((x - y).abs() < 1e-5);
    }
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
