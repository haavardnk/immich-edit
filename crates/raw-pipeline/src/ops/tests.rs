use super::LinearImage;
use super::*;
use crate::edits::{
    BasicEdits, ColorEdits, ColorGradeEdits, ColorGradeRegion, DetailEdits, Edits, GeometryEdits,
    HslBand, HslEdits, RetouchMode, RetouchStroke, ToneEdits, Vec2f,
};

fn solid_image(w: usize, h: usize, rgb: [f32; 3]) -> LinearImage {
    let mut buf = Vec::with_capacity(w * h * 3);
    for _ in 0..w * h {
        buf.extend_from_slice(&rgb);
    }
    LinearImage::new(buf, w, h)
}

fn ctx() -> OpContext {
    OpContext {
        render: RenderContext {
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            cam_to_srgb: crate::color::identity_3x3(),
            is_raw: false,
            capture_sigma: None,
            preview_mode: crate::frame::PreviewMode::None,
            roi: None,
            dcp: None,
        },
        scratch: OpScratch::default(),
    }
}

mod basic;
mod color;
mod detail;
mod registry;
mod strokes;
mod tone;
