use crate::color::{SceneWhite, Temperature};
use crate::dcp::DcpProfile;
use crate::edits::{Edits, WhiteBalance};
use crate::frame::{FrameMeta, RawFrame};
use crate::geom::{GeometryTransform, display_uv_to_mask_uv};
use crate::ops::lens_distortion::LensWarpParams;
use crate::sensor_sample::{
    camera_wb_coeffs, demosaic_patch, display_rgb, geometry_transform, sample_raw_bilinear,
};
use serde::Serialize;

mod auto;

pub use auto::auto_white_balance;

const SAMPLE_RADIUS: i32 = 2;
const MIN_SIGNAL: f32 = 1e-4;
const NEUTRAL_EPS: f64 = 1e-4;
const SOLVER_STEPS: usize = 40;
const CLIP_LEVEL: f32 = 0.95;
const MAX_CHROMA_SPREAD: f64 = 0.2;
const DISPLAY_SEARCH: f64 = 100.0;
const CAMERA_SEARCH: f64 = 400.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct AsShotWhite {
    pub raw: bool,
    pub kelvin: f64,
    pub tint: f64,
}

pub fn as_shot_white(meta: &FrameMeta, edits: &Edits, profile: Option<&DcpProfile>) -> AsShotWhite {
    let white = crate::dcp::setup::resolve(meta, edits, profile).white;
    let Temperature { kelvin, tint } = white.as_shot();
    AsShotWhite {
        raw: matches!(white, SceneWhite::Camera(_)),
        kelvin: kelvin.round(),
        tint: tint.round(),
    }
}

pub fn sample_white_balance(
    frame: &RawFrame,
    edits: &Edits,
    profile: Option<&DcpProfile>,
    u: f32,
    v: f32,
) -> Option<WhiteBalance> {
    let (wb, m, white) = scene_color(frame, edits, profile);
    let (px, py) = display_uv_to_sensor_px(frame, edits, u, v)?;
    let patch = demosaic_patch(frame, px, py, SAMPLE_RADIUS as usize);
    let (source, px, py) = match &patch {
        Some((patch, x, y)) => (patch, *x, *y),
        None => (frame, px, py),
    };

    let raws: Vec<[f32; 3]> = (-SAMPLE_RADIUS..=SAMPLE_RADIUS)
        .flat_map(|dy| (-SAMPLE_RADIUS..=SAMPLE_RADIUS).map(move |dx| (dy, dx)))
        .filter_map(|(dy, dx)| sample_raw_bilinear(source, px + dx as f32, py + dy as f32))
        .collect();
    if raws.is_empty() || raws.iter().flatten().any(|c| *c >= CLIP_LEVEL) {
        return None;
    }
    let samples: Vec<[f32; 3]> = raws.iter().map(|raw| display_rgb(*raw, wb, m)).collect();
    if chroma_spread(&samples)? > MAX_CHROMA_SPREAD {
        return None;
    }
    let mut acc = [0.0f64; 3];
    for s in &samples {
        acc[0] += s[0] as f64;
        acc[1] += s[1] as f64;
        acc[2] += s[2] as f64;
    }
    solve_neutral(&white, mean_rgb(acc, samples.len() as u32))
}

fn scene_color(
    frame: &RawFrame,
    edits: &Edits,
    profile: Option<&DcpProfile>,
) -> ([f32; 3], [[f32; 3]; 3], SceneWhite) {
    let setup = crate::dcp::setup::resolve(&frame.meta, edits, profile);
    (
        camera_wb_coeffs(frame.meta.wb_coeffs),
        setup.cam_to_srgb,
        setup.white,
    )
}

fn chroma_spread(samples: &[[f32; 3]]) -> Option<f64> {
    let ratios: Vec<[f64; 2]> = samples
        .iter()
        .map(|s| {
            if s.iter().any(|c| *c <= MIN_SIGNAL) {
                return None;
            }
            Some([
                (s[0] as f64 / s[1] as f64).ln(),
                (s[2] as f64 / s[1] as f64).ln(),
            ])
        })
        .collect::<Option<_>>()?;
    let n = ratios.len() as f64;
    let spread = |axis: usize| {
        let mean = ratios.iter().map(|r| r[axis]).sum::<f64>() / n;
        (ratios.iter().map(|r| (r[axis] - mean).powi(2)).sum::<f64>() / n).sqrt()
    };
    Some(spread(0).max(spread(1)))
}

fn mean_rgb(acc: [f64; 3], count: u32) -> [f32; 3] {
    let n = count as f64;
    [
        (acc[0] / n) as f32,
        (acc[1] / n) as f32,
        (acc[2] / n) as f32,
    ]
}

fn solve_neutral(white: &SceneWhite, rgb: [f32; 3]) -> Option<WhiteBalance> {
    if rgb.iter().any(|c| !c.is_finite()) || rgb[1] <= MIN_SIGNAL {
        return None;
    }
    let limit = if matches!(white, SceneWhite::Camera(_)) {
        CAMERA_SEARCH
    } else {
        DISPLAY_SEARCH
    };
    let candidate = |temp: f64, tint: f64| offset_white(white, temp, tint);
    let mut temp = 0.0f64;
    let mut tint = 0.0f64;
    for _ in 0..SOLVER_STEPS {
        let f = residual(white, rgb, &candidate(temp, tint))?;
        if f[0].abs() < NEUTRAL_EPS && f[1].abs() < NEUTRAL_EPS {
            break;
        }
        let h = 0.5;
        let ft = residual(white, rgb, &candidate(temp + h, tint))?;
        let fi = residual(white, rgb, &candidate(temp, tint + h))?;
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
        temp = (temp - d_temp.clamp(-40.0, 40.0)).clamp(-limit, limit);
        tint = (tint - d_tint.clamp(-40.0, 40.0)).clamp(-limit, limit);
    }
    Some(rounded(candidate(temp, tint)))
}

fn offset_white(white: &SceneWhite, temp: f64, tint: f64) -> WhiteBalance {
    match white {
        SceneWhite::Camera(_) => WhiteBalance::from(white.as_shot().shifted(temp, tint)),
        SceneWhite::Display => WhiteBalance::Relative { temp, tint },
    }
}

fn rounded(wb: WhiteBalance) -> WhiteBalance {
    match wb {
        WhiteBalance::Absolute { kelvin, tint } => WhiteBalance::Absolute {
            kelvin: kelvin.round(),
            tint: tint.round(),
        },
        WhiteBalance::Relative { temp, tint } => WhiteBalance::Relative {
            temp: temp.round(),
            tint: tint.round(),
        },
    }
}

fn residual(white: &SceneWhite, rgb: [f32; 3], wb: &WhiteBalance) -> Option<[f64; 2]> {
    let m = white.user_matrix(wb);
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
    let warp = LensWarpParams::from_edits(
        &edits.lens,
        frame.meta.width as u32,
        frame.meta.height as u32,
    );
    let (sx, sy) = warp.output_px_to_source_px(px, py);
    (sx >= 0.0 && sy >= 0.0 && sx <= w - 1.0 && sy <= h - 1.0).then_some((sx, sy))
}

#[cfg(test)]
mod tests;
