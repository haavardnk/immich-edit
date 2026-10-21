use raw_pipeline::frame::{FrameId, FrameMeta};

use super::*;

const SIZE: usize = 32;

fn uniform_frame(rgb: [f32; 3]) -> Arc<RawFrame> {
    Arc::new(RawFrame {
        meta: FrameMeta {
            width: SIZE,
            height: SIZE,
            wb_coeffs: [1.0; 4],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: String::new(),
        bps: 16,
        data: rgb.repeat(SIZE * SIZE),
        cpp: 3,
        exif: None,
        id: FrameId::fresh(),
    })
}

fn sample_at(u: f32, v: f32, w: f32, h: f32) -> WhiteBalanceSample {
    WhiteBalanceSample {
        u,
        v,
        w,
        h,
        edits: Edits::default(),
    }
}

fn centre() -> WhiteBalanceSample {
    sample_at(0.5, 0.5, 0.0, 0.0)
}

#[test]
fn samples_outside_the_unit_square_are_rejected() {
    for (u, v, w, h, ok) in [
        (0.0, 1.0, 0.0, 0.0, true),
        (0.5, 0.5, 0.0, 0.0, true),
        (0.2, 0.3, 0.8, 0.7, true),
        (0.25, 0.5, 0.5, 0.0, true),
        (1.5, 0.5, 0.0, 0.0, false),
        (0.5, -0.1, 0.0, 0.0, false),
        (f32::NAN, 0.5, 0.0, 0.0, false),
        (0.5, 0.5, 0.6, 0.1, false),
        (0.5, 0.5, 0.1, -0.1, false),
        (0.5, 0.5, f32::NAN, 0.1, false),
        (0.5, 0.5, 0.1, f32::INFINITY, false),
    ] {
        assert_eq!(
            sample_at(u, v, w, h).validate().is_ok(),
            ok,
            "({u}, {v}, {w}, {h})"
        );
    }
}

#[tokio::test]
async fn a_warm_sample_solves_to_a_cooler_temperature() {
    for req in [centre(), sample_at(0.25, 0.25, 0.5, 0.5)] {
        let solved = sample(uniform_frame([0.6, 0.45, 0.3]), req, None)
            .await
            .expect("warm sample solves");
        let WhiteBalance::Relative { temp, .. } = solved else {
            panic!("display white solves relative, got {solved:?}");
        };
        assert!(temp < 0.0, "got {solved:?}");
    }
}

#[tokio::test]
async fn a_black_frame_has_no_usable_colour() {
    let sampled = sample(uniform_frame([0.0; 3]), centre(), None).await;
    let automatic = auto(uniform_frame([0.0; 3]), Edits::default(), None).await;
    assert!(matches!(sampled, Err(AppError::Unprocessable(_))));
    assert!(matches!(automatic, Err(AppError::Unprocessable(_))));
}
