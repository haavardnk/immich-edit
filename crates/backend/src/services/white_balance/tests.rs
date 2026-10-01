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

fn centre() -> SamplePoint {
    SamplePoint {
        u: 0.5,
        v: 0.5,
        edits: Edits::default(),
    }
}

#[test]
fn sample_points_outside_the_unit_square_are_rejected() {
    for (u, v, ok) in [
        (0.0, 1.0, true),
        (0.5, 0.5, true),
        (1.5, 0.5, false),
        (0.5, -0.1, false),
        (f32::NAN, 0.5, false),
    ] {
        let point = SamplePoint {
            u,
            v,
            edits: Edits::default(),
        };
        assert_eq!(point.validate().is_ok(), ok, "({u}, {v})");
    }
}

#[tokio::test]
async fn a_warm_sample_solves_to_a_cooler_temperature() {
    let solved = sample(uniform_frame([0.6, 0.45, 0.3]), centre(), None)
        .await
        .expect("warm sample solves");
    assert!(solved.wb_temp < 0.0, "got {solved:?}");
}

#[tokio::test]
async fn a_black_frame_has_no_usable_colour() {
    let sampled = sample(uniform_frame([0.0; 3]), centre(), None).await;
    let automatic = auto(uniform_frame([0.0; 3]), Edits::default(), None).await;
    assert!(matches!(sampled, Err(AppError::Unprocessable(_))));
    assert!(matches!(automatic, Err(AppError::Unprocessable(_))));
}
