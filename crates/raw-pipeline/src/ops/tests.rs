use super::LinearImage;
use super::*;
use crate::edits::{BasicEdits, CropRect, Edits, GeometryEdits, ToneEdits};

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
fn contrast_pivots_around_half() {
    let mut img = solid_image(1, 1, [0.5, 0.5, 0.5]);
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
    assert!((img.rgb[0] - 0.5).abs() < 1e-5);
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
fn highlights_lift_bright_pixels() {
    let mut img = solid_image(1, 1, [0.8, 0.8, 0.8]);
    let edits = Edits {
        tone: ToneEdits {
            highlights: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    highlights_shadows::HighlightsShadowsOp
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
    highlights_shadows::HighlightsShadowsOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > 0.2);
}

#[test]
fn white_balance_temp_warms() {
    let mut img = solid_image(1, 1, [0.5, 0.5, 0.5]);
    let edits = Edits {
        basic: BasicEdits {
            wb_temp: 100.0,
            ..Default::default()
        },
        ..Default::default()
    };
    white_balance::WhiteBalanceOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert!(img.rgb[0] > img.rgb[2]);
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
fn geometry_crop_reduces_dims() {
    let mut img = solid_image(10, 10, [0.5, 0.5, 0.5]);
    let edits = Edits {
        geometry: GeometryEdits {
            crop: Some(CropRect {
                x: 0.0,
                y: 0.0,
                width: 0.5,
                height: 0.5,
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    geometry::GeometryOp
        .apply_cpu(&mut img, &ctx(), &edits)
        .unwrap();
    assert_eq!(img.width, 5);
    assert_eq!(img.height, 5);
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
    assert_eq!(active, vec!["white_balance"]);
}
