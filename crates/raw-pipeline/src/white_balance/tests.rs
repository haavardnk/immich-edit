use super::*;
use crate::edits::{CropRect, GeometryEdits};
use crate::frame::FrameMeta;

fn make_frame(w: usize, h: usize, rgb: [f32; 3]) -> RawFrame {
    make_frame_with(w, h, |_, _| rgb)
}

fn make_frame_with<F: Fn(usize, usize) -> [f32; 3]>(w: usize, h: usize, f: F) -> RawFrame {
    let mut data = Vec::with_capacity(w * h * 3);
    for (y, x) in (0..h).flat_map(|y| (0..w).map(move |x| (y, x))) {
        data.extend_from_slice(&f(x, y));
    }
    RawFrame {
        meta: FrameMeta {
            width: w,
            height: h,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: String::new(),
        bps: 16,
        data,
        cpp: 3,
        exif: None,
    }
}

fn neutralised(rgb: [f32; 3], temp: f64, tint: f64) -> [f32; 3] {
    let m = user_wb_matrix(temp, tint);
    [
        m[0][0] * rgb[0] + m[0][1] * rgb[1] + m[0][2] * rgb[2],
        m[1][0] * rgb[0] + m[1][1] * rgb[1] + m[1][2] * rgb[2],
        m[2][0] * rgb[0] + m[2][1] * rgb[1] + m[2][2] * rgb[2],
    ]
}

fn spread(rgb: [f32; 3]) -> f32 {
    let max = rgb[0].max(rgb[1]).max(rgb[2]);
    let min = rgb[0].min(rgb[1]).min(rgb[2]);
    (max - min) / max.max(1e-6)
}

#[test]
fn neutral_sample_solves_to_no_correction() {
    let (temp, tint) = solve_neutral([0.4, 0.4, 0.4]).expect("neutral grey solves");
    assert_eq!((temp, tint), (0.0, 0.0));
}

#[test]
fn solved_pair_neutralises_the_sample() {
    for rgb in [
        [0.50, 0.40, 0.25],
        [0.20, 0.30, 0.45],
        [0.35, 0.40, 0.35],
        [0.06, 0.05, 0.04],
    ] {
        let (temp, tint) = solve_neutral(rgb).expect("cast solves");
        let out = neutralised(rgb, temp, tint);
        assert!(
            spread(out) < 0.03,
            "rgb {rgb:?} -> ({temp}, {tint}) left spread {}",
            spread(out)
        );
    }
}

#[test]
fn warm_cast_cools_and_blue_cast_warms() {
    let (warm, _) = solve_neutral([0.6, 0.45, 0.3]).expect("warm solves");
    let (cool, _) = solve_neutral([0.3, 0.45, 0.6]).expect("cool solves");
    assert!(
        warm < 0.0,
        "warm sample should lower temperature, got {warm}"
    );
    assert!(
        cool > 0.0,
        "cool sample should raise temperature, got {cool}"
    );
}

#[test]
fn black_and_non_finite_samples_do_not_solve() {
    assert!(solve_neutral([0.0, 0.0, 0.0]).is_none());
    assert!(solve_neutral([0.4, f32::NAN, 0.4]).is_none());
}

#[test]
fn sampling_a_patch_neutralises_that_patch() {
    let cast = [0.55, 0.40, 0.28];
    let frame = make_frame_with(64, 64, |x, _| if x < 32 { cast } else { [0.1, 0.4, 0.1] });
    let (temp, tint) =
        sample_white_balance(&frame, &Edits::default(), 0.15, 0.5).expect("patch solves");
    assert!(spread(neutralised(cast, temp, tint)) < 0.03);
}

#[test]
fn sampling_outside_the_frame_returns_none() {
    let frame = make_frame(16, 16, [0.4, 0.4, 0.4]);
    assert!(sample_white_balance(&frame, &Edits::default(), 1.4, 0.5).is_none());
}

#[test]
fn sample_uses_display_coordinates_after_crop() {
    let cast = [0.55, 0.40, 0.28];
    let frame = make_frame_with(64, 64, |x, _| if x < 32 { [0.1, 0.4, 0.1] } else { cast });
    let edits = Edits {
        geometry: GeometryEdits {
            crop: Some(CropRect {
                x: 0.5,
                y: 0.0,
                w: 0.5,
                h: 1.0,
            }),
            ..Default::default()
        },
        ..Default::default()
    };
    let (temp, tint) =
        sample_white_balance(&frame, &edits, 0.5, 0.5).expect("cropped patch solves");
    assert!(spread(neutralised(cast, temp, tint)) < 0.03);
}

#[test]
fn auto_white_balance_neutralises_a_grey_world() {
    let cast = [0.5, 0.4, 0.3];
    let frame = make_frame_with(64, 64, |x, y| {
        let scale = 0.4 + 0.6 * ((x + y) % 8) as f32 / 8.0;
        [cast[0] * scale, cast[1] * scale, cast[2] * scale]
    });
    let (temp, tint) = auto_white_balance(&frame, &Edits::default()).expect("grey world solves");
    assert!(spread(neutralised(cast, temp, tint)) < 0.03);
}

#[test]
fn auto_white_balance_ignores_a_small_specular_highlight() {
    let cast = [0.5, 0.4, 0.3];
    let frame = make_frame_with(64, 64, |x, y| {
        if x < 6 && y < 6 {
            [40.0, 0.2, 0.2]
        } else {
            cast
        }
    });
    let (temp, tint) = auto_white_balance(&frame, &Edits::default()).expect("clipped world solves");
    assert!(spread(neutralised(cast, temp, tint)) < 0.03);
}

#[test]
fn auto_white_balance_on_black_frame_returns_none() {
    let frame = make_frame(32, 32, [0.0, 0.0, 0.0]);
    assert!(auto_white_balance(&frame, &Edits::default()).is_none());
}
