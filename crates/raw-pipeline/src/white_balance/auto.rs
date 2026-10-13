use super::{CLIP_LEVEL, MIN_SIGNAL, offset_white, oriented_geometry, rounded, scene_color};
use crate::color::{SceneWhite, Temperature, inverse_3x3, mat3_vec};
use crate::dcp::DcpProfile;
use crate::edits::{Edits, WhiteBalance};
use crate::frame::RawFrame;
use crate::geom::mask_uv_to_display_uv;
use crate::sensor_sample::{
    SAMPLE_TARGET, decimate_mosaic, display_rgb, sample_raw_bilinear, sensor_to_oriented_uv,
};

const TEMP_STEPS: i32 = 50;
const TEMP_STEP: f64 = 2.0;
const TINT_STEPS: i32 = 10;
const TINT_STEP: f64 = 3.0;
const STOPS_PER_TEMP: f64 = 1.5 / 100.0;
const WARMER_PRIOR_STOPS: f64 = 0.2;
const COOLER_PRIOR_STOPS: f64 = 0.12;
const TINT_PRIOR: f64 = 10.0;
const NEUTRAL_RADIUS: f64 = 0.05;
const BIN: f64 = 0.01;
const CHROMA_RANGE: f64 = 2.0;
const DARK_FRACTION: f32 = 0.02;
const BRIGHT_QUANTILE: f64 = 0.98;
const SUPPORT_SCALE: f64 = 0.04;
const PEAK_SHARE: f64 = 0.8;
const D65_KELVIN: f64 = 6504.0;
const WARM_ADAPTATION: f64 = 0.8;

struct Sample {
    chroma: [f64; 2],
    weight: f64,
}

pub fn auto_white_balance(
    frame: &RawFrame,
    edits: &Edits,
    profile: Option<&DcpProfile>,
) -> Option<WhiteBalance> {
    let decimated = decimate_mosaic(frame);
    let frame = decimated.as_ref().unwrap_or(frame);
    let (wb, m, white) = scene_color(frame, edits, profile);
    let samples = lit_samples(sample_frame(frame, edits, wb, m));
    if samples.is_empty() {
        return None;
    }
    let side = (2.0 * CHROMA_RANGE / BIN) as usize;
    let mut votes = vec![0.0f64; side * side];
    for s in &samples {
        if let Some(i) = bin(s.chroma, side) {
            votes[i] += s.weight;
        }
    }
    let total: f64 = samples.iter().map(|s| s.weight).sum();
    let scores: Vec<(f64, f64, f64)> = (-TEMP_STEPS..=TEMP_STEPS)
        .flat_map(|t| (-TINT_STEPS..=TINT_STEPS).map(move |i| (t, i)))
        .filter_map(|(t, i)| {
            let temp = t as f64 * TEMP_STEP;
            let tint = i as f64 * TINT_STEP;
            let support = support(&votes, side, neutral_chroma(&white, temp, tint)?) / total;
            Some((support * prior(temp, tint), temp, tint))
        })
        .collect();
    let best = scores.iter().map(|s| s.0).fold(0.0, f64::max);
    let peak: Vec<&(f64, f64, f64)> = scores.iter().filter(|s| s.0 >= PEAK_SHARE * best).collect();
    let mass = peak.iter().map(|s| s.0).sum::<f64>().max(f64::MIN_POSITIVE);
    let temp = peak.iter().map(|s| s.0 * s.1).sum::<f64>() / mass;
    let tint = peak.iter().map(|s| s.0 * s.2).sum::<f64>() / mass;
    let confidence = 1.0 - (-best / SUPPORT_SCALE).exp();
    Some(rounded(match offset_white(&white, temp, tint) {
        WhiteBalance::Absolute { kelvin, tint } => {
            let shot = white.as_shot();
            let shot_mired = 1.0e6 / shot.kelvin;
            let mired = shot_mired + confidence * (adapted_mired(1.0e6 / kelvin) - shot_mired);
            WhiteBalance::from(
                Temperature {
                    kelvin: 1.0e6 / mired,
                    tint: shot.tint + confidence * (tint - shot.tint),
                }
                .clamped(),
            )
        }
        WhiteBalance::Relative { temp, tint } => WhiteBalance::Relative {
            temp: confidence * temp,
            tint: confidence * tint,
        },
    }))
}

fn adapted_mired(mired: f64) -> f64 {
    let d65 = 1.0e6 / D65_KELVIN;
    if mired <= d65 {
        return mired;
    }
    d65 + WARM_ADAPTATION * (mired - d65)
}

fn sample_frame(frame: &RawFrame, edits: &Edits, wb: [f32; 3], m: [[f32; 3]; 3]) -> Vec<[f32; 3]> {
    let geom = oriented_geometry(frame, edits);
    let total = frame.meta.width * frame.meta.height;
    let step = (total / SAMPLE_TARGET).max(1);
    (0..total)
        .step_by(step)
        .filter_map(|i| {
            let px = (i % frame.meta.width) as f32;
            let py = (i / frame.meta.width) as f32;
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
                    return None;
                }
            }
            let raw = sample_raw_bilinear(frame, px, py)?;
            if raw.iter().any(|c| *c >= CLIP_LEVEL) {
                return None;
            }
            Some(display_rgb(raw, wb, m))
        })
        .collect()
}

fn lit_samples(rgb: Vec<[f32; 3]>) -> Vec<Sample> {
    let luma = |c: &[f32; 3]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
    let mut sorted: Vec<f32> = rgb.iter().map(luma).collect();
    if sorted.is_empty() {
        return Vec::new();
    }
    sorted.sort_by(f32::total_cmp);
    let bright = sorted[((sorted.len() - 1) as f64 * BRIGHT_QUANTILE) as usize];
    let floor = (bright * DARK_FRACTION).max(MIN_SIGNAL);
    rgb.into_iter()
        .filter(|c| luma(c) >= floor && c.iter().all(|v| *v > MIN_SIGNAL))
        .map(|c| Sample {
            chroma: [
                (c[0] as f64 / c[1] as f64).ln(),
                (c[2] as f64 / c[1] as f64).ln(),
            ],
            weight: luma(&c) as f64,
        })
        .collect()
}

fn neutral_chroma(white: &SceneWhite, temp: f64, tint: f64) -> Option<[f64; 2]> {
    let undo = inverse_3x3(white.user_matrix(&offset_white(white, temp, tint)))?;
    let n = mat3_vec(&undo, [1.0; 3]);
    if n.iter().any(|c| *c <= MIN_SIGNAL) {
        return None;
    }
    Some([
        (n[0] as f64 / n[1] as f64).ln(),
        (n[2] as f64 / n[1] as f64).ln(),
    ])
}

fn prior(temp: f64, tint: f64) -> f64 {
    let width = if temp < 0.0 {
        COOLER_PRIOR_STOPS
    } else {
        WARMER_PRIOR_STOPS
    };
    let stops = temp * STOPS_PER_TEMP / width;
    let tint = tint / TINT_PRIOR;
    (-0.5 * (stops * stops + tint * tint)).exp()
}

fn bin(chroma: [f64; 2], side: usize) -> Option<usize> {
    let x = ((chroma[0] + CHROMA_RANGE) / BIN).floor();
    let y = ((chroma[1] + CHROMA_RANGE) / BIN).floor();
    let limit = side as f64;
    (x >= 0.0 && y >= 0.0 && x < limit && y < limit).then(|| y as usize * side + x as usize)
}

fn support(votes: &[f64], side: usize, n: [f64; 2]) -> f64 {
    let reach = (NEUTRAL_RADIUS / BIN).ceil() as i64;
    let cx = ((n[0] + CHROMA_RANGE) / BIN).floor() as i64;
    let cy = ((n[1] + CHROMA_RANGE) / BIN).floor() as i64;
    let side = side as i64;
    (cy - reach..=cy + reach)
        .flat_map(|y| (cx - reach..=cx + reach).map(move |x| (x, y)))
        .filter(|&(x, y)| x >= 0 && y >= 0 && x < side && y < side)
        .filter(|&(x, y)| {
            let center = [
                (x as f64 + 0.5) * BIN - CHROMA_RANGE,
                (y as f64 + 0.5) * BIN - CHROMA_RANGE,
            ];
            distance(center, n) <= NEUTRAL_RADIUS
        })
        .map(|(x, y)| votes[(y * side + x) as usize])
        .sum()
}

fn distance(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
