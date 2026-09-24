use crate::color::user_wb_matrix;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::geom::{GeometryTransform, display_uv_to_mask_uv, mask_uv_to_display_uv};
use crate::ops::lens_distortion::{distortion_coeffs, distortion_zoom, output_px_to_source_px};
use crate::sensor_sample::{
    SAMPLE_TARGET, decimate_mosaic, display_color, display_rgb, geometry_transform,
    sample_raw_bilinear, sensor_to_oriented_uv,
};

const SAMPLE_RADIUS: i32 = 2;
const MIN_SIGNAL: f32 = 1e-4;
const NEUTRAL_EPS: f64 = 1e-4;
const SOLVER_STEPS: usize = 40;
const GREY_WORLD_CLIP: f64 = 0.02;

pub fn sample_white_balance(frame: &RawFrame, edits: &Edits, u: f32, v: f32) -> Option<(f64, f64)> {
    let decimated = decimate_mosaic(frame);
    let frame = decimated.as_ref().unwrap_or(frame);
    let (wb, m) = display_color(frame);
    let (px, py) = display_uv_to_sensor_px(frame, edits, u, v)?;

    let mut acc = [0.0f64; 3];
    let mut count = 0u32;
    for (dy, dx) in (-SAMPLE_RADIUS..=SAMPLE_RADIUS)
        .flat_map(|dy| (-SAMPLE_RADIUS..=SAMPLE_RADIUS).map(move |dx| (dy, dx)))
    {
        let Some(raw) = sample_raw_bilinear(frame, px + dx as f32, py + dy as f32) else {
            continue;
        };
        let rgb = display_rgb(raw, wb, m);
        acc[0] += rgb[0] as f64;
        acc[1] += rgb[1] as f64;
        acc[2] += rgb[2] as f64;
        count += 1;
    }
    if count == 0 {
        return None;
    }
    solve_neutral(mean_rgb(acc, count))
}

pub fn auto_white_balance(frame: &RawFrame, edits: &Edits) -> Option<(f64, f64)> {
    let decimated = decimate_mosaic(frame);
    let frame = decimated.as_ref().unwrap_or(frame);
    let (wb, m) = display_color(frame);
    let geom = oriented_geometry(frame, edits);
    let total = frame.meta.width * frame.meta.height;
    let step = (total / SAMPLE_TARGET).max(1);

    let mut samples: Vec<[f32; 3]> = Vec::new();
    let mut i = 0;
    while i < total {
        let px = (i % frame.meta.width) as f32;
        let py = (i / frame.meta.width) as f32;
        i += step;
        if let Some(geom) = &geom {
            let (u, v) = sensor_to_oriented_uv(
                px + 0.5,
                py + 0.5,
                frame.meta.width,
                frame.meta.height,
                frame.meta.orientation,
            );
            let d = mask_uv_to_display_uv(geom, [u, v]);
            if d[0] < 0.0 || d[0] > 1.0 || d[1] < 0.0 || d[1] > 1.0 {
                continue;
            }
        }
        let Some(raw) = sample_raw_bilinear(frame, px, py) else {
            continue;
        };
        let rgb = display_rgb(raw, wb, m);
        if rgb[0] + rgb[1] + rgb[2] > MIN_SIGNAL {
            samples.push(rgb);
        }
    }
    if samples.is_empty() {
        return None;
    }

    let mut luma: Vec<f32> = samples.iter().map(|s| s[0] + s[1] + s[2]).collect();
    luma.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let cut_index = ((luma.len() as f64 * (1.0 - GREY_WORLD_CLIP)) as usize).min(luma.len() - 1);
    let cut = luma[cut_index];

    let mut acc = [0.0f64; 3];
    let mut count = 0u32;
    for s in samples.iter().filter(|s| s[0] + s[1] + s[2] <= cut) {
        acc[0] += s[0] as f64;
        acc[1] += s[1] as f64;
        acc[2] += s[2] as f64;
        count += 1;
    }
    if count == 0 {
        return None;
    }
    solve_neutral(mean_rgb(acc, count))
}

fn mean_rgb(acc: [f64; 3], count: u32) -> [f32; 3] {
    let n = count as f64;
    [
        (acc[0] / n) as f32,
        (acc[1] / n) as f32,
        (acc[2] / n) as f32,
    ]
}

fn solve_neutral(rgb: [f32; 3]) -> Option<(f64, f64)> {
    if rgb.iter().any(|c| !c.is_finite()) || rgb[1] <= MIN_SIGNAL {
        return None;
    }
    let mut temp = 0.0f64;
    let mut tint = 0.0f64;
    for _ in 0..SOLVER_STEPS {
        let f = residual(rgb, temp, tint)?;
        if f[0].abs() < NEUTRAL_EPS && f[1].abs() < NEUTRAL_EPS {
            break;
        }
        let h = 0.5;
        let ft = residual(rgb, temp + h, tint)?;
        let fi = residual(rgb, temp, tint + h)?;
        let j = [
            [(ft[0] - f[0]) / h, (fi[0] - f[0]) / h],
            [(ft[1] - f[1]) / h, (fi[1] - f[1]) / h],
        ];
        let det = j[0][0] * j[1][1] - j[0][1] * j[1][0];
        if det.abs() < 1e-12 {
            break;
        }
        let d_temp = (f[0] * j[1][1] - f[1] * j[0][1]) / det;
        let d_tint = (f[1] * j[0][0] - f[0] * j[1][0]) / det;
        temp = (temp - d_temp.clamp(-40.0, 40.0)).clamp(-100.0, 100.0);
        tint = (tint - d_tint.clamp(-40.0, 40.0)).clamp(-100.0, 100.0);
    }
    Some((temp.round(), tint.round()))
}

fn residual(rgb: [f32; 3], temp: f64, tint: f64) -> Option<[f64; 2]> {
    let m = user_wb_matrix(temp, tint);
    let out = [
        m[0][0] * rgb[0] + m[0][1] * rgb[1] + m[0][2] * rgb[2],
        m[1][0] * rgb[0] + m[1][1] * rgb[1] + m[1][2] * rgb[2],
        m[2][0] * rgb[0] + m[2][1] * rgb[1] + m[2][2] * rgb[2],
    ];
    if out.iter().any(|c| *c <= MIN_SIGNAL) {
        return None;
    }
    Some([
        (out[0] as f64 / out[1] as f64).ln(),
        (out[2] as f64 / out[1] as f64).ln(),
    ])
}

fn oriented_geometry(frame: &RawFrame, edits: &Edits) -> Option<GeometryTransform> {
    let (transpose, _, _) = frame.meta.orientation;
    let (ow, oh) = if transpose {
        (frame.meta.height, frame.meta.width)
    } else {
        (frame.meta.width, frame.meta.height)
    };
    geometry_transform(edits, ow as u32, oh as u32)
}

fn display_uv_to_sensor_px(frame: &RawFrame, edits: &Edits, u: f32, v: f32) -> Option<(f32, f32)> {
    if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
        return None;
    }
    let oriented = match oriented_geometry(frame, edits) {
        Some(geom) => display_uv_to_mask_uv(&geom, [u, v]),
        None => [u, v],
    };
    if !(0.0..=1.0).contains(&oriented[0]) || !(0.0..=1.0).contains(&oriented[1]) {
        return None;
    }

    let w = frame.meta.width as f32;
    let h = frame.meta.height as f32;
    let (transpose, flip_h, flip_v) = frame.meta.orientation;
    let (mut px, mut py) = if transpose {
        (oriented[1] * w, oriented[0] * h)
    } else {
        (oriented[0] * w, oriented[1] * h)
    };
    if flip_h {
        px = w - px;
    }
    if flip_v {
        py = h - py;
    }
    px = px.clamp(0.0, w - 1.0);
    py = py.clamp(0.0, h - 1.0);

    if !edits.lens.distortion_active() {
        return Some((px, py));
    }
    let (k1, k2, k3) = distortion_coeffs(&edits.lens);
    let zoom = distortion_zoom(&edits.lens);
    let (sx, sy) = output_px_to_source_px(
        k1,
        k2,
        k3,
        zoom,
        frame.meta.width as u32,
        frame.meta.height as u32,
        px,
        py,
    );
    (sx >= 0.0 && sy >= 0.0 && sx <= w - 1.0 && sy <= h - 1.0).then_some((sx, sy))
}

#[cfg(test)]
mod tests;
