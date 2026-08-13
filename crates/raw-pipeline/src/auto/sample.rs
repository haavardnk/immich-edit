use super::SAMPLE_TARGET;
use crate::edits::{CropRect, Edits};
use crate::frame::{OrientFlips, RawFrame};
use crate::geom::GeometryTransform;

fn camera_wb_coeffs(raw: [f32; 4]) -> [f32; 3] {
    if raw[0] == 0.0 && raw[1] == 0.0 && raw[2] == 0.0 {
        return [1.0, 1.0, 1.0];
    }
    if raw[1] > 0.0 {
        return [raw[0] / raw[1], 1.0, raw[2] / raw[1]];
    }
    [raw[0], raw[1], raw[2]]
}

fn parse_cfa(cfa_pattern: &str) -> [u8; 4] {
    let mut cfa = *b"RGGB";
    for (i, b) in cfa_pattern.bytes().take(4).enumerate() {
        cfa[i] = b;
    }
    cfa
}

fn cfa_block(cfa_pattern: &str) -> (usize, Vec<usize>) {
    let to_channel = |b: u8| match b {
        b'R' => 0usize,
        b'B' => 2,
        _ => 1,
    };
    match crate::cpu::demosaic::parse_xtrans(cfa_pattern) {
        Some(pattern) => (6, pattern.iter().map(|b| to_channel(*b)).collect()),
        None => (
            2,
            parse_cfa(cfa_pattern)
                .iter()
                .map(|b| to_channel(*b))
                .collect(),
        ),
    }
}

fn mosaic_block_rgb(
    data: &[f32],
    w: usize,
    dim: usize,
    table: &[usize],
    bx: usize,
    by: usize,
) -> [f32; 3] {
    let mut acc = [0.0f32; 3];
    let mut counts = [0.0f32; 3];
    for (dy, dx) in (0..dim).flat_map(|dy| (0..dim).map(move |dx| (dy, dx))) {
        let ch = table[dy * dim + dx];
        acc[ch] += data[(by + dy) * w + bx + dx];
        counts[ch] += 1.0;
    }
    for ch in 0..3 {
        if counts[ch] > 0.0 {
            acc[ch] /= counts[ch];
        }
    }
    acc
}
pub(super) fn display_color(frame: &RawFrame) -> ([f32; 3], [[f32; 3]; 3]) {
    let wb = camera_wb_coeffs(frame.wb_coeffs);
    let xyz_to_cam =
        crate::color::resolve_xyz_to_cam(&frame.color_matrices, frame.wb_coeffs, frame.xyz_to_cam);
    if !frame.is_raw || crate::color::is_unusable_matrix(&xyz_to_cam) {
        return (wb, crate::color::identity_3x3());
    }
    (wb, crate::color::cam_to_srgb_matrix(xyz_to_cam))
}

pub(super) fn display_rgb(raw: [f32; 3], wb: [f32; 3], m: [[f32; 3]; 3]) -> [f32; 3] {
    let r = (raw[0] * wb[0]).max(0.0);
    let g = (raw[1] * wb[1]).max(0.0);
    let b = (raw[2] * wb[2]).max(0.0);
    [
        (m[0][0] * r + m[0][1] * g + m[0][2] * b).max(0.0),
        (m[1][0] * r + m[1][1] * g + m[1][2] * b).max(0.0),
        (m[2][0] * r + m[2][1] * g + m[2][2] * b).max(0.0),
    ]
}

pub(super) fn develop_luma(r: f32, g: f32, b: f32) -> f32 {
    crate::tone::apply_display_luma([r, g, b])
}
pub(super) fn sample_raw_bilinear(frame: &RawFrame, x: f32, y: f32) -> Option<[f32; 3]> {
    let w = frame.width as i32;
    let h = frame.height as i32;
    if w <= 0 || h <= 0 || frame.cpp < 3 {
        return None;
    }
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let tx = x - xi as f32;
    let ty = y - yi as f32;
    let stride = frame.cpp;
    let load = |ix: i32, iy: i32| -> [f32; 3] {
        let cx = ix.clamp(0, w - 1) as usize;
        let cy = iy.clamp(0, h - 1) as usize;
        let off = (cy * frame.width + cx) * stride;
        [frame.data[off], frame.data[off + 1], frame.data[off + 2]]
    };
    let c00 = load(xi, yi);
    let c10 = load(xi + 1, yi);
    let c01 = load(xi, yi + 1);
    let c11 = load(xi + 1, yi + 1);
    let mix = |a: f32, b: f32, t: f32| a * (1.0 - t) + b * t;
    let mut out = [0.0f32; 3];
    for c in 0..3 {
        let a = mix(c00[c], c10[c], tx);
        let b = mix(c01[c], c11[c], tx);
        out[c] = mix(a, b, ty);
    }
    Some(out)
}

pub(super) fn sensor_to_oriented_uv(
    px: f32,
    py: f32,
    w: usize,
    h: usize,
    orient: OrientFlips,
) -> (f32, f32) {
    let (t, hf, vf) = orient;
    let wf = w as f32;
    let hf32 = h as f32;
    let mut px2 = px;
    let mut py2 = py;
    if hf {
        px2 = wf - px2;
    }
    if vf {
        py2 = hf32 - py2;
    }
    let (ox, oy, ow, oh) = if t {
        (py2, px2, hf32, wf)
    } else {
        (px2, py2, wf, hf32)
    };
    (ox / ow, oy / oh)
}

pub(super) fn geometry_transform(
    edits: &Edits,
    oriented_w: u32,
    oriented_h: u32,
) -> Option<GeometryTransform> {
    let g = &edits.geometry;
    let crop = g.crop.unwrap_or(CropRect {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    });
    let t = GeometryTransform {
        input_w: oriented_w,
        input_h: oriented_h,
        rotate_quarter: g.rotate,
        flip_h: g.flip_h,
        flip_v: g.flip_v,
        angle_deg: g.rotate_angle,
        crop,
        perspective_forward: g.perspective_forward(),
        perspective_inverse: g.perspective_inverse(),
        output_w: oriented_w,
        output_h: oriented_h,
    };
    if t.is_identity() { None } else { Some(t) }
}
pub(super) fn decimate_mosaic(frame: &RawFrame) -> Option<RawFrame> {
    if frame.cpp != 1 || frame.cfa_pattern.is_empty() {
        return None;
    }
    let (dim, table) = cfa_block(&frame.cfa_pattern);
    let bw = frame.width / dim;
    let bh = frame.height / dim;
    if bw == 0 || bh == 0 {
        return None;
    }
    let stride = ((bw * bh) / SAMPLE_TARGET).isqrt().clamp(1, bw.min(bh));
    let out_w = bw / stride;
    let out_h = bh / stride;
    let mut data = Vec::with_capacity(out_w * out_h * 3);
    for (oy, ox) in (0..out_h).flat_map(|oy| (0..out_w).map(move |ox| (oy, ox))) {
        let rgb = mosaic_block_rgb(
            &frame.data,
            frame.width,
            dim,
            &table,
            ox * stride * dim,
            oy * stride * dim,
        );
        data.extend_from_slice(&rgb);
    }
    Some(RawFrame {
        width: out_w,
        height: out_h,
        cfa_pattern: String::new(),
        bps: frame.bps,
        wb_coeffs: frame.wb_coeffs,
        xyz_to_cam: frame.xyz_to_cam,
        color_matrices: frame.color_matrices.clone(),
        data,
        cpp: 3,
        orientation: frame.orientation,
        is_raw: frame.is_raw,
        capture_sigma: None,
        model: frame.model.clone(),
        exif: None,
    })
}
