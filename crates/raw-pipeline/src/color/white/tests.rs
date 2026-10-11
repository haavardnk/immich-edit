use std::sync::Arc;

use super::*;
use crate::dcp::{HsvEncoding, HueSatMap};
use crate::edits::{BasicEdits, Edits, WhiteBalance};
use crate::frame::FrameMeta;

const CM_A: [[f32; 3]; 3] = [
    [0.7234, -0.1413, -0.0600],
    [-0.3631, 1.1150, 0.2850],
    [-0.0382, 0.1335, 0.6437],
];
const CM_D65: [[f32; 3]; 3] = [
    [0.6722, -0.0635, -0.0963],
    [-0.4287, 1.2460, 0.2028],
    [-0.0908, 0.2162, 0.5668],
];
const FM_A: [[f32; 3]; 3] = [
    [0.5303, 0.2410, 0.1929],
    [0.4200, 0.3700, 0.2100],
    [0.0412, 0.2062, 0.5775],
];
const FM_D65: [[f32; 3]; 3] = [
    [0.4821, 0.2893, 0.1928],
    [0.4000, 0.3500, 0.2500],
    [0.0825, 0.2475, 0.4949],
];
const D55: [f32; 2] = [0.3324, 0.3474];

fn rel(temp: f64, tint: f64) -> WhiteBalance {
    WhiteBalance::Relative { temp, tint }
}

fn dual_profile(forward: bool) -> DcpProfile {
    let table = |hue: f32| {
        Some(Arc::new(HueSatMap {
            hue_div: 2,
            sat_div: 1,
            val_div: 1,
            encoding: HsvEncoding::Linear,
            data: vec![[hue, 1.0, 1.0]; 2],
        }))
    };
    DcpProfile {
        name: None,
        copyright: None,
        unique_camera_model: None,
        calibration_illuminant1: 17,
        calibration_illuminant2: Some(21),
        color_matrix1: CM_A,
        color_matrix2: Some(CM_D65),
        forward_matrix1: forward.then_some(FM_A),
        forward_matrix2: forward.then_some(FM_D65),
        huesatmap1: table(-10.0),
        huesatmap2: table(10.0),
        look_table: None,
        tone_curve: None,
        baseline_exposure_offset: 0.0,
        default_black_render: 0,
        embed_policy: 0,
    }
}

fn shot_at(profile: &DcpProfile, xy: [f32; 2]) -> (CameraWhite, [f32; 4]) {
    let neutral = CameraWhite::from_dcp(profile, DcpIlluminant::Interpolated, [1.0; 4]).neutral(xy);
    let wb = [1.0 / neutral[0], 1.0, 1.0 / neutral[2], 1.0];
    (
        CameraWhite::from_dcp(profile, DcpIlluminant::Interpolated, wb),
        wb,
    )
}

fn develop(camera: &CameraWhite, wb: [f32; 4], cam: [f32; 3]) -> [f32; 3] {
    mat3_vec(&camera.cam_to_srgb(), [0, 1, 2].map(|c| cam[c] * wb[c]))
}

fn meta(wb_coeffs: [f32; 4]) -> FrameMeta {
    FrameMeta {
        width: 4,
        height: 4,
        wb_coeffs,
        xyz_to_cam: [[0.0; 3]; 4],
        color_matrices: Vec::new(),
        orientation: (false, false, false),
        is_raw: true,
        capture_sigma: None,
        model: String::new(),
    }
}

#[test]
fn user_white_balance_matches_shooting_at_that_white() {
    let patches = [
        [0.2, 0.3, 0.1],
        [0.05, 0.2, 0.4],
        [0.5, 0.25, 0.15],
        [0.3, 0.3, 0.3],
    ];
    for (forward, wb) in [
        (false, rel(-60.0, 0.0)),
        (true, rel(-60.0, 0.0)),
        (false, rel(35.0, -25.0)),
        (true, rel(35.0, -25.0)),
        (true, rel(-90.0, 40.0)),
        (
            true,
            WhiteBalance::Absolute {
                kelvin: 3200.0,
                tint: 12.0,
            },
        ),
    ] {
        let profile = dual_profile(forward);
        let (camera, wb0) = shot_at(&profile, D55);
        let target = target(camera.as_shot, &wb);
        let (reshot, wb1) = shot_at(&profile, target);
        let case = format!("forward {forward}, {wb:?}");
        assert!(
            (reshot.as_shot[0] - target[0]).abs() + (reshot.as_shot[1] - target[1]).abs() < 1e-4,
            "{case}: {:?} vs {target:?}",
            reshot.as_shot
        );
        let user = camera.user_matrix(&wb);
        for cam in patches {
            let edited = mat3_vec(&user, develop(&camera, wb0, cam));
            let expected = develop(&reshot, wb1, cam);
            let err = (0..3)
                .map(|c| (edited[c] - expected[c]).abs())
                .fold(0.0, f32::max);
            assert!(err < 2e-3, "{case}: {cam:?} -> {edited:?} vs {expected:?}");
        }
    }
}

#[test]
fn dcp_base_table_follows_user_white() {
    let profile = dual_profile(false);
    let (camera, wb0) = shot_at(&profile, D55);
    let (_, wb1) = shot_at(&profile, target(camera.as_shot, &rel(-60.0, 0.0)));
    let warmed = Edits {
        basic: BasicEdits {
            white_balance: rel(-60.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    };
    let hue = |wb: [f32; 4], edits: &Edits| {
        let setup = crate::dcp::setup::resolve(&meta(wb), edits, Some(&profile));
        setup
            .resolved
            .and_then(|r| r.base_table.clone())
            .map(|t| t.data[0][0])
    };
    let edited = hue(wb0, &warmed).expect("base table");
    let reshot = hue(wb1, &Edits::default()).expect("base table");
    let shot = hue(wb0, &Edits::default()).expect("base table");
    assert!((edited - reshot).abs() < 0.05, "{edited} vs {reshot}");
    assert!((edited - shot).abs() > 5.0, "{edited} vs {shot}");
}

#[test]
fn zero_edit_is_identity_and_slider_signs_hold() {
    let whites = [
        SceneWhite::Display,
        SceneWhite::Camera(shot_at(&dual_profile(false), D55).0),
        SceneWhite::Camera(shot_at(&dual_profile(true), D55).0),
    ];
    for white in whites {
        let still = white.user_matrix(&WhiteBalance::AS_SHOT);
        let off = (0..9)
            .map(|i| (still[i / 3][i % 3] - identity_3x3()[i / 3][i % 3]).abs())
            .fold(0.0, f32::max);
        assert!(off < 1e-4, "{white:?}: {still:?}");
        let shot = WhiteBalance::from(white.as_shot());
        let pinned = white.user_matrix(&shot);
        let off = (0..9)
            .map(|i| (pinned[i / 3][i % 3] - identity_3x3()[i / 3][i % 3]).abs())
            .fold(0.0, f32::max);
        assert!(off < 1e-4, "{white:?} pinned: {pinned:?}");
        let grey = |temp: f64, tint: f64| mat3_vec(&white.user_matrix(&rel(temp, tint)), [0.5; 3]);
        let [r, _, b] = grey(50.0, 0.0);
        assert!(r > b, "warm {r} {b}");
        let [r, _, b] = grey(-50.0, 0.0);
        assert!(r < b, "cool {r} {b}");
        let [r, g, b] = grey(0.0, -50.0);
        assert!(g > (r + b) / 2.0, "green {r} {g} {b}");
        let [r, g, b] = grey(0.0, 50.0);
        assert!(g < (r + b) / 2.0, "magenta {r} {g} {b}");
    }
}

#[test]
fn temperature_range_spans_candlelight_to_shade() {
    let kelvin = |xy: [f32; 2]| Temperature::from_xy(xy.map(f64::from)).kelvin;
    let warmest = kelvin(target(D55, &rel(-100.0, 0.0)));
    let coolest = kelvin(target(D55, &rel(100.0, 0.0)));
    assert!(warmest < 2100.0, "{warmest}");
    assert!(coolest > 12000.0, "{coolest}");
}

#[test]
fn extreme_whites_keep_their_cast_direction() {
    for (forward, kelvin, tint) in [
        (false, 2000.0, 150.0),
        (true, 2000.0, 150.0),
        (true, 2000.0, -150.0),
        (true, 50000.0, 150.0),
        (true, 50000.0, -150.0),
    ] {
        let camera = shot_at(&dual_profile(forward), D55).0;
        let [r, g, b] = mat3_vec(
            &camera.user_matrix(&WhiteBalance::Absolute { kelvin, tint }),
            [0.5; 3],
        );
        let case = format!("forward {forward}, {kelvin} K {tint}: {r} {g} {b}");
        assert!([r, g, b].iter().all(|c| c.is_finite()), "{case}");
        assert_eq!(b > r, kelvin < 5000.0, "{case}");
        assert_eq!(g < (r + b) / 2.0, tint > 0.0, "{case}");
    }
}

#[test]
fn forward_matrix_maps_camera_neutral_to_white() {
    let camera = CameraWhite::from_dcp(
        &dual_profile(true),
        DcpIlluminant::Interpolated,
        [2.0, 1.0, 1.5, 1.0],
    );
    let out = mat3_vec(&camera.cam_to_srgb(), [1.0; 3]);
    assert!(out.iter().all(|c| (c - 1.0).abs() < 1e-3), "{out:?}");
}
