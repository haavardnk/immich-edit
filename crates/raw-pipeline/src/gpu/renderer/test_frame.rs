use crate::frame::{FrameId, FrameMeta, RawFrame};

const W: usize = 96;
const H: usize = 64;

pub(super) fn checker_frame() -> RawFrame {
    let data = (0..W * H)
        .flat_map(|i| {
            let x = i % W;
            let y = i / W;
            let checker = if (x / 3 + y / 3) % 2 == 0 { 0.7 } else { 0.2 };
            [checker, x as f32 / W as f32, y as f32 / H as f32]
        })
        .collect();
    RawFrame {
        meta: FrameMeta {
            width: W,
            height: H,
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
        embedded_lens: Default::default(),
        id: FrameId::fresh(),
    }
}
