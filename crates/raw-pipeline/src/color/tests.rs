use super::dcp::*;
use super::hsv::*;
use super::matrix::*;
use crate::dcp::{DcpIlluminant, DcpProfile, HsvEncoding, HueSatMap};

const MATRIX_A: [[f32; 3]; 4] = [
    [0.8, 0.1, 0.1],
    [0.05, 0.9, 0.05],
    [0.1, 0.2, 0.7],
    [0.0, 0.0, 0.0],
];
const MATRIX_D65: [[f32; 3]; 4] = [
    [0.6, 0.3, 0.1],
    [0.1, 0.8, 0.1],
    [0.05, 0.1, 0.85],
    [0.0, 0.0, 0.0],
];

#[test]
fn display_p3_roundtrip_is_identity() {
    for c in [[0.2, 0.5, 0.8], [1.0, 0.0, 0.0], [0.3, 0.9, 0.1]] {
        let back = display_p3_to_srgb_lin(srgb_lin_to_display_p3(c));
        for i in 0..3 {
            assert!((back[i] - c[i]).abs() < 1e-4, "roundtrip drift at {i}");
        }
    }
}

#[test]
fn srgb_red_maps_inside_display_p3() {
    let p3 = srgb_lin_to_display_p3([1.0, 0.0, 0.0]);
    assert!(p3.iter().all(|&v| (0.0..=1.0).contains(&v)));
    assert!(p3[0] < 1.0 && p3[0] > 0.8);
}

#[test]
fn display_p3_red_is_outside_srgb() {
    let srgb = display_p3_to_srgb_lin([1.0, 0.0, 0.0]);
    assert!(srgb[0] > 1.0 || srgb[1] < 0.0 || srgb[2] < 0.0);
}

#[test]
fn interpolate_at_low_cct_returns_warm_matrix() {
    let matrices = vec![(2856.0, MATRIX_A), (6504.0, MATRIX_D65)];
    let result = interpolate_xyz_to_cam(&matrices, 2856.0);
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                (result[i][j] - MATRIX_A[i][j]).abs() < 1e-5,
                "mismatch at [{i}][{j}]"
            );
        }
    }
}

#[test]
fn interpolate_at_high_cct_returns_cool_matrix() {
    let matrices = vec![(2856.0, MATRIX_A), (6504.0, MATRIX_D65)];
    let result = interpolate_xyz_to_cam(&matrices, 6504.0);
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                (result[i][j] - MATRIX_D65[i][j]).abs() < 1e-5,
                "mismatch at [{i}][{j}]"
            );
        }
    }
}

#[test]
fn interpolate_midpoint_blends() {
    let matrices = vec![(2856.0, MATRIX_A), (6504.0, MATRIX_D65)];
    let mid_cct = 4000.0;
    let result = interpolate_xyz_to_cam(&matrices, mid_cct);
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                result[i][j] > MATRIX_A[i][j].min(MATRIX_D65[i][j]) - 1e-5
                    && result[i][j] < MATRIX_A[i][j].max(MATRIX_D65[i][j]) + 1e-5,
                "out of range at [{i}][{j}]: {}",
                result[i][j]
            );
        }
    }
}

#[test]
fn interpolate_single_matrix_returns_it() {
    let matrices = vec![(6504.0, MATRIX_D65)];
    let result = interpolate_xyz_to_cam(&matrices, 4000.0);
    assert_eq!(result, MATRIX_D65);
}

#[test]
fn estimate_cct_returns_valid_range() {
    let matrix: [[f32; 3]; 4] = [
        [0.8, 0.1, 0.1],
        [0.05, 0.9, 0.05],
        [0.1, 0.2, 0.7],
        [0.0, 0.0, 0.0],
    ];
    let wb = [2.0, 1.0, 1.5, 1.0];
    let cct = estimate_scene_cct(wb, &matrix);
    assert!((2000.0..=25000.0).contains(&cct), "cct={cct} out of range");
}

fn make_dcp(fm: Option<[[f32; 3]; 3]>) -> DcpProfile {
    DcpProfile {
        name: None,
        copyright: None,
        unique_camera_model: None,
        calibration_illuminant1: 21,
        calibration_illuminant2: None,
        color_matrix1: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        color_matrix2: None,
        forward_matrix1: fm,
        forward_matrix2: None,
        huesatmap1: None,
        huesatmap2: None,
        look_table: None,
        tone_curve: None,
        baseline_exposure_offset: 0.0,
        default_black_render: 0,
        embed_policy: 0,
    }
}

#[test]
fn dcp_forward_matrix_maps_neutral_to_white() {
    // ForwardMatrix rows sum to D50 whitepoint XYZ, per DNG spec.
    let fm = [
        [0.9642 * 0.5, 0.9642 * 0.3, 0.9642 * 0.2],
        [1.0 * 0.4, 1.0 * 0.35, 1.0 * 0.25],
        [0.8249 * 0.1, 0.8249 * 0.3, 0.8249 * 0.6],
    ];
    let profile = make_dcp(Some(fm));
    let m = dcp_cam_to_srgb(&profile, [2.0, 1.0, 1.5, 1.0], DcpIlluminant::Interpolated);
    let out = mat3_vec(&m, [1.0, 1.0, 1.0]);
    for c in out {
        assert!((c - 1.0).abs() < 1e-3, "neutral -> {out:?}");
    }
}

#[test]
fn prophoto_roundtrip_is_identity() {
    let to_pp = srgb_lin_to_prophoto_matrix();
    let from_pp = prophoto_to_srgb_lin_matrix();
    for rgb in [[0.2, 0.5, 0.8], [0.9, 0.1, 0.3], [0.5, 0.5, 0.5]] {
        let round = mat3_vec(&from_pp, mat3_vec(&to_pp, rgb));
        for (a, b) in round.iter().zip(rgb.iter()) {
            assert!((a - b).abs() < 1e-3, "roundtrip {round:?} vs {rgb:?}");
        }
    }
}

#[test]
fn huesat_identity_map_preserves_color() {
    let map = HueSatMap {
        hue_div: 4,
        sat_div: 3,
        val_div: 1,
        encoding: HsvEncoding::Linear,
        data: vec![[0.0, 1.0, 1.0]; 12],
    };
    let to_pp = srgb_lin_to_prophoto_matrix();
    let from_pp = prophoto_to_srgb_lin_matrix();
    for rgb in [[0.3, 0.5, 0.7], [0.8, 0.2, 0.4]] {
        let out = apply_huesat(&map, &to_pp, &from_pp, rgb);
        for (a, b) in out.iter().zip(rgb.iter()) {
            assert!((a - b).abs() < 2e-3, "huesat identity {out:?} vs {rgb:?}");
        }
    }
}

#[test]
fn srgb_encoded_huesat_identity_preserves_color() {
    let map = HueSatMap {
        hue_div: 4,
        sat_div: 3,
        val_div: 3,
        encoding: HsvEncoding::Srgb,
        data: vec![[0.0, 1.0, 1.0]; 36],
    };
    let to_pp = srgb_lin_to_prophoto_matrix();
    let from_pp = prophoto_to_srgb_lin_matrix();
    let rgb = [0.3, 0.5, 0.7];
    let out = apply_huesat(&map, &to_pp, &from_pp, rgb);
    if out
        .iter()
        .zip(rgb.iter())
        .any(|(a, b)| (a - b).abs() >= 2e-3)
    {
        panic!("sRGB huesat identity {out:?} vs {rgb:?}");
    }
}

#[test]
fn dcp_finish_preserves_neutral_axis() {
    let map = HueSatMap {
        hue_div: 4,
        sat_div: 3,
        val_div: 3,
        encoding: HsvEncoding::Srgb,
        data: vec![[30.0, 1.5, 0.8]; 36],
    };
    let to_pp = srgb_lin_to_prophoto_matrix();
    let from_pp = prophoto_to_srgb_lin_matrix();
    let out = apply_dcp_finish(
        Some(&map),
        Some(&DCP_FALLBACK_TONE_CURVE),
        &to_pp,
        &from_pp,
        [0.4, 0.4, 0.4],
    );
    if (out[0] - out[1]).abs() > 2e-3 || (out[1] - out[2]).abs() > 2e-3 {
        panic!("DCP finish shifted neutral axis: {out:?}");
    }
}

#[test]
fn dcp_finish_orders_looktable_before_tone_curve() {
    let look = HueSatMap {
        hue_div: 3,
        sat_div: 3,
        val_div: 1,
        encoding: HsvEncoding::Srgb,
        data: vec![[24.0, 1.3, 0.85]; 9],
    };
    let curve = [[0.0f32, 0.0], [0.25, 0.4], [0.6, 0.55], [1.0, 1.0]];
    let to_pp = srgb_lin_to_prophoto_matrix();
    let from_pp = prophoto_to_srgb_lin_matrix();
    let rgb = [0.62, 0.28, 0.16];

    let got = apply_dcp_finish(Some(&look), Some(&curve), &to_pp, &from_pp, rgb);

    let pp = mat3_vec(&to_pp, rgb);
    let correct = mat3_vec(
        &from_pp,
        apply_profile_tone_curve(&curve, apply_huesat_in_space(&look, pp, true)),
    );
    let wrong = mat3_vec(
        &from_pp,
        apply_huesat_in_space(&look, apply_profile_tone_curve(&curve, pp), true),
    );

    if (0..3).any(|i| (got[i] - correct[i]).abs() > 1e-6) {
        panic!("finish is not look-before-tone: {got:?} vs {correct:?}");
    }
    if (0..3).all(|i| (correct[i] - wrong[i]).abs() < 1e-4) {
        panic!("stage order is not observable; test cannot guard ordering");
    }
    let golden = [0.5450712, 0.38464665, 0.07066608];
    if (0..3).any(|i| (got[i] - golden[i]).abs() > 1e-4) {
        panic!("DCP finish golden drift: {got:?}");
    }
}

#[test]
fn dcp_base_table_golden() {
    let map = HueSatMap {
        hue_div: 2,
        sat_div: 2,
        val_div: 1,
        encoding: HsvEncoding::Linear,
        data: vec![[18.0, 0.75, 1.08]; 4],
    };
    let to_pp = srgb_lin_to_prophoto_matrix();
    let from_pp = prophoto_to_srgb_lin_matrix();
    let got = apply_huesat(&map, &to_pp, &from_pp, [0.5, 0.24, 0.11]);
    let golden = [0.4618022, 0.35953194, 0.17766672];
    if (0..3).any(|i| (got[i] - golden[i]).abs() > 1e-4) {
        panic!("DCP base table golden drift: {got:?}");
    }
}

#[test]
fn profile_tone_curve_preserves_hsv_hue() {
    let rgb = [0.72, 0.31, 0.18];
    let before = rgb_to_hsv(rgb);
    let out = apply_profile_tone_curve(&DCP_FALLBACK_TONE_CURVE, rgb);
    let after = rgb_to_hsv(out);
    if (before[0] - after[0]).abs() > 1e-5 {
        panic!("profile tone shifted hue: {before:?} -> {after:?}");
    }
}

#[test]
fn tone_curve_is_monotonic_and_bounded() {
    let mut prev = -1.0;
    for i in 0..=20 {
        let x = i as f32 / 20.0;
        let y = eval_tone_curve(&DCP_FALLBACK_TONE_CURVE, x);
        assert!((0.0..=1.0).contains(&y));
        assert!(y >= prev - 1e-6, "not monotonic at {x}");
        prev = y;
    }
    assert!((eval_tone_curve(&DCP_FALLBACK_TONE_CURVE, 0.0)).abs() < 1e-6);
    assert!((eval_tone_curve(&DCP_FALLBACK_TONE_CURVE, 1.0) - 1.0).abs() < 1e-6);
}
