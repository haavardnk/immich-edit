use crate::edits::{CropRect, Edits};
use crate::frame::RawFrame;

pub(super) struct ProcessGeom {
    pub display: (u32, u32),
    pub oriented: (u32, u32),
    pub source: (u32, u32),
    pub crop: CropRect,
    pub cos_a: f32,
    pub sin_a: f32,
    pub bw: f32,
    pub bh: f32,
    pub persp_rows: [f32; 12],
    pub orient_packed: u32,
    pub geom_warps: bool,
}

pub(super) fn process_geom(frame: &RawFrame, edits: &Edits, work_dims: (u32, u32)) -> ProcessGeom {
    let (sensor_w, sensor_h) = work_dims;
    let display = if frame.orientation.0 {
        (sensor_h, sensor_w)
    } else {
        (sensor_w, sensor_h)
    };
    let oriented = match edits.geometry.rotate {
        90 | 270 => (display.1, display.0),
        _ => display,
    };

    let full_display = if frame.orientation.0 {
        (frame.height as u32, frame.width as u32)
    } else {
        (frame.width as u32, frame.height as u32)
    };
    let source = match edits.geometry.rotate {
        90 | 270 => (full_display.1, full_display.0),
        _ => full_display,
    };

    let crop = edits.geometry.crop.unwrap_or(CropRect::full());
    let angle = edits.geometry.rotate_angle;
    let bbox = crate::geom::rotated_bbox(oriented.0 as f32, oriented.1 as f32, angle);
    let a_rad = crate::geom::deg_to_rad(angle);
    let perspective_inverse = edits.geometry.perspective_inverse();

    let (ot, oh_h, oh_v) = frame.orientation;
    ProcessGeom {
        display,
        oriented,
        source,
        crop,
        cos_a: a_rad.cos(),
        sin_a: a_rad.sin(),
        bw: bbox.w,
        bh: bbox.h,
        persp_rows: crate::perspective::mat3_rows(&perspective_inverse),
        orient_packed: (oh_h as u32) | ((oh_v as u32) << 1) | ((ot as u32) << 2),
        geom_warps: !crop.is_full()
            || angle.abs() > 1e-4
            || perspective_inverse != crate::perspective::IDENTITY,
    }
}

pub(super) fn compute_out_dims(
    frame: &RawFrame,
    edits: &Edits,
    src_dims: (u32, u32),
    max_edge: u32,
) -> (u32, u32) {
    crate::geom::display_out_dims(frame.orientation, edits, src_dims, max_edge)
}

pub(super) fn crop_px(frame: &RawFrame, edits: &Edits, src_dims: (u32, u32)) -> (u32, u32) {
    crate::geom::display_crop_px(frame.orientation, edits, src_dims)
}
