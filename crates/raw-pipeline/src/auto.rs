use crate::cpu::pipeline::default_tone;
use crate::edits::{BasicEdits, Edits, ToneEdits};
use crate::frame::RawFrame;

const SAMPLE_TARGET: usize = 200_000;
const HIST_BINS: usize = 256;

fn develop(v: f32) -> f32 {
    default_tone(v)
}

struct Stats {
    hist: [u32; HIST_BINS],
    total: u32,
    mean_sat: f32,
}

fn hist_percentile(hist: &[u32; HIST_BINS], total: u32, p: f64) -> usize {
    let target = (total as f64 * p) as u32;
    let mut cumulative = 0u32;
    for (i, &v) in hist.iter().enumerate() {
        cumulative += v;
        if cumulative >= target {
            return i;
        }
    }
    HIST_BINS - 1
}

fn hist_fraction_above(hist: &[u32; HIST_BINS], total: u32, threshold: usize) -> f64 {
    let sum: u32 = hist[threshold..].iter().sum();
    sum as f64 / total.max(1) as f64
}

fn hist_fraction_below(hist: &[u32; HIST_BINS], total: u32, threshold: usize) -> f64 {
    let sum: u32 = hist[..=threshold.min(HIST_BINS - 1)].iter().sum();
    sum as f64 / total.max(1) as f64
}

fn collect_stats(frame: &RawFrame) -> Option<Stats> {
    let pixel_count = frame.data.len() / 3;
    if pixel_count == 0 {
        return None;
    }
    let step = (pixel_count / SAMPLE_TARGET).max(1);

    let wb = frame.wb_coeffs;
    let g = wb[1].max(1e-6);
    let raw_scale = [wb[0] / g, 1.0, wb[2] / g];
    let max_scale = raw_scale[0].max(raw_scale[1]).max(raw_scale[2]);
    let norm = if max_scale > 1.0 {
        1.0 / max_scale
    } else {
        1.0
    };
    let scale = [
        raw_scale[0] * norm,
        raw_scale[1] * norm,
        raw_scale[2] * norm,
    ];

    let mut hist = [0u32; HIST_BINS];
    let mut sat_sum: f64 = 0.0;
    let mut sat_n: u32 = 0;
    let mut total = 0u32;

    let mut i = 0;
    while i < pixel_count {
        let off = i * 3;
        let r = (frame.data[off] * scale[0]).max(0.0);
        let gv = (frame.data[off + 1] * scale[1]).max(0.0);
        let b = (frame.data[off + 2] * scale[2]).max(0.0);

        let y_linear = 0.2126 * r + 0.7152 * gv + 0.0722 * b;
        let y_srgb = develop(y_linear);
        let bin = (y_srgb * 255.0).round().clamp(0.0, 255.0) as usize;
        hist[bin] += 1;
        total += 1;

        if y_srgb > 0.05 && y_srgb < 0.95 {
            let mx = r.max(gv).max(b);
            let mn = r.min(gv).min(b);
            if mx > 1e-4 {
                let s = (mx - mn) / mx;
                sat_sum += s as f64;
                sat_n += 1;
            }
        }
        i += step;
    }

    let mean_sat = if sat_n > 0 {
        (sat_sum / sat_n as f64) as f32
    } else {
        0.0
    };
    Some(Stats {
        hist,
        total,
        mean_sat,
    })
}

pub fn auto_adjust(frame: &RawFrame) -> Edits {
    let Some(s) = collect_stats(frame) else {
        return Edits::default();
    };
    if s.total == 0 {
        return Edits::default();
    }

    let p01 = hist_percentile(&s.hist, s.total, 0.01);
    let p50 = hist_percentile(&s.hist, s.total, 0.50);
    let p99 = hist_percentile(&s.hist, s.total, 0.99);

    let black_point = p01;
    let white_point = p99;
    let range = (white_point as f64 - black_point as f64).max(1.0);

    let highlight_frac = hist_fraction_above(&s.hist, s.total, 240);
    let clipped_frac = hist_fraction_above(&s.hist, s.total, 250);

    let mut exposure_ev = (128.0 - p50 as f64) * 0.008;
    if white_point > 245 || highlight_frac > 0.02 || clipped_frac > 0.005 {
        exposure_ev = exposure_ev.min(0.0);
    }
    exposure_ev = exposure_ev.clamp(-2.0, 2.0);

    let mut contrast = 0.0f64;
    if range < 200.0 {
        contrast = ((200.0 / range) - 1.0) * 8.0;
    }
    if highlight_frac > 0.02 {
        contrast *= 0.5;
    }
    contrast = contrast.clamp(-30.0, 30.0);

    let shadow_frac = hist_fraction_below(&s.hist, s.total, 32);
    let mut shadows = 0.0f64;
    if shadow_frac > 0.05 {
        shadows = (shadow_frac * 40.0).min(35.0);
    }

    let mut highlights = 0.0f64;
    if highlight_frac > 0.02 {
        highlights = -(highlight_frac * 120.0).min(70.0);
    }

    let simulated_p01 = (p01 as f64 + exposure_ev * 20.0).clamp(0.0, 255.0);
    let simulated_p99 = (p99 as f64 + exposure_ev * 20.0).clamp(0.0, 255.0);
    let blacks = -(simulated_p01 * 0.2).clamp(-15.0, 15.0);
    let whites = ((simulated_p99 - 255.0) * 0.15).clamp(-25.0, 0.0);

    let mut vibrance = 0.0f64;
    if s.mean_sat < 0.20 {
        vibrance = (0.20 - s.mean_sat) as f64 * 120.0;
    }
    vibrance = vibrance.clamp(0.0, 40.0);

    Edits {
        basic: BasicEdits {
            exposure_ev,
            contrast,
            vibrance,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights,
            shadows,
            blacks,
            whites,
        },
        color: Default::default(),
        geometry: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame_luma(luma: f32, w: usize, h: usize) -> RawFrame {
        let mut data = Vec::with_capacity(w * h * 3);
        for _ in 0..(w * h) {
            data.push(luma);
            data.push(luma);
            data.push(luma);
        }
        RawFrame {
            width: w,
            height: h,
            cfa_pattern: String::new(),
            bps: 16,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            data,
            cpp: 3,
            orientation: (false, false, false),
            is_raw: false,
            exif: None,
        }
    }

    #[test]
    fn well_exposed_image_small_exposure() {
        let f = make_frame_luma(0.18, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev.abs() > 0.5 {
            panic!(
                "expected small exposure for well-exposed image, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn very_dark_image_gets_positive_exposure() {
        let f = make_frame_luma(0.01, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev <= 0.3 {
            panic!(
                "expected positive exposure for dark image, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn bright_image_gets_negative_or_zero_exposure() {
        let f = make_frame_luma(0.8, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev > 0.1 {
            panic!(
                "expected non-positive exposure for bright image, got {}",
                e.basic.exposure_ev
            );
        }
    }

    #[test]
    fn clipped_highlights_get_recovery() {
        let mut data: Vec<f32> = Vec::with_capacity(64 * 64 * 3);
        for i in 0..(64 * 64) {
            let v = if i % 4 == 0 { 0.99 } else { 0.3 };
            data.push(v);
            data.push(v);
            data.push(v);
        }
        let f = RawFrame {
            width: 64,
            height: 64,
            cfa_pattern: String::new(),
            bps: 16,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            data,
            cpp: 3,
            orientation: (false, false, false),
            is_raw: false,
            exif: None,
        };
        let e = auto_adjust(&f);
        if e.tone.highlights >= 0.0 {
            panic!("expected highlight recovery, got {}", e.tone.highlights);
        }
    }

    #[test]
    fn exposure_capped_with_bright_highlights() {
        let f = make_frame_luma(0.95, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev > 0.0 {
            panic!(
                "expected non-positive exposure with clipped highlights, got {}",
                e.basic.exposure_ev
            );
        }
    }
}
