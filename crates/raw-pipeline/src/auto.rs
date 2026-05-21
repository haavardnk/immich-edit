use crate::edits::{BasicEdits, Edits, ToneEdits};
use crate::frame::RawFrame;

const SAMPLE_TARGET: usize = 200_000;

struct Stats {
    luma_pct: [f32; 13],
    mean_sat: f32,
    high_sat_frac: f32,
}

const PCTS: [f32; 13] = [
    0.001, 0.005, 0.01, 0.02, 0.05, 0.25, 0.50, 0.75, 0.95, 0.98, 0.99, 0.995, 0.999,
];

fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn collect_stats(frame: &RawFrame) -> Option<Stats> {
    let total = frame.width * frame.height;
    if total == 0 {
        return None;
    }
    let step = (total / SAMPLE_TARGET).max(1);

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

    let cap = total / step + 1;
    let mut lumas: Vec<f32> = Vec::with_capacity(cap);
    let mut sat_sum: f64 = 0.0;
    let mut sat_n: u32 = 0;
    let mut high_sat: u32 = 0;

    let mut i = 0;
    while i < total {
        let off = i * 3;
        let r = (frame.data[off] * scale[0]).max(0.0);
        let gv = (frame.data[off + 1] * scale[1]).max(0.0);
        let b = (frame.data[off + 2] * scale[2]).max(0.0);
        let y = 0.2126 * r + 0.7152 * gv + 0.0722 * b;
        lumas.push(y.clamp(0.0, 4.0));

        if y > 0.02 && y < 0.98 {
            let mx = r.max(gv).max(b);
            let mn = r.min(gv).min(b);
            if mx > 1e-4 {
                let s = (mx - mn) / mx;
                sat_sum += s as f64;
                sat_n += 1;
                if s > 0.3 {
                    high_sat += 1;
                }
            }
        }
        i += step;
    }

    lumas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut luma_pct = [0.0f32; 13];
    for (idx, p) in PCTS.iter().enumerate() {
        luma_pct[idx] = percentile(&lumas, *p);
    }
    let mean_sat = if sat_n > 0 {
        (sat_sum / sat_n as f64) as f32
    } else {
        0.0
    };
    let high_sat_frac = if sat_n > 0 {
        high_sat as f32 / sat_n as f32
    } else {
        0.0
    };
    Some(Stats {
        luma_pct,
        mean_sat,
        high_sat_frac,
    })
}

pub fn auto_adjust(frame: &RawFrame) -> Edits {
    let Some(s) = collect_stats(frame) else {
        return Edits::default();
    };
    let p001 = s.luma_pct[0];
    let p02 = s.luma_pct[3];
    let p05 = s.luma_pct[4];
    let p25 = s.luma_pct[5];
    let p50 = s.luma_pct[6];
    let p75 = s.luma_pct[7];
    let p95 = s.luma_pct[8];
    let p98 = s.luma_pct[9];
    let p995 = s.luma_pct[11];
    let p999 = s.luma_pct[12];

    let target_median = 0.18f32;
    let mut exposure_ev = if p50 > 1e-4 {
        (target_median / p50).log2().clamp(-2.5, 2.5)
    } else {
        2.5
    };

    let dynamic_range = (p95 - p05).max(0.0);
    if dynamic_range > 0.85 {
        exposure_ev = (exposure_ev - 0.2).max(-2.5);
    }

    let gain = 2f32.powf(exposure_ev);
    let lift = |x: f32| (x * gain).min(4.0);
    let p001e = lift(p001);
    let p02e = lift(p02);
    let p05e = lift(p05);
    let p25e = lift(p25);
    let p75e = lift(p75);
    let p98e = lift(p98);
    let p995e = lift(p995);
    let p999e = lift(p999);

    let highlights = if p98e > 0.85 {
        (-90.0 * ((p98e - 0.85) / 0.30)).clamp(-90.0, 0.0)
    } else {
        0.0
    };

    let shadows = if p05e < 0.18 {
        (80.0 * ((0.18 - p05e) / 0.18)).clamp(0.0, 80.0)
    } else if p05e > 0.35 {
        (-40.0 * ((p05e - 0.35) / 0.30)).clamp(-40.0, 0.0)
    } else {
        0.0
    };

    let whites = if p999e > 0.96 {
        (-60.0 * ((p999e - 0.96) / 0.20)).clamp(-60.0, 0.0)
    } else if p995e < 0.85 {
        (40.0 * ((0.85 - p995e) / 0.40)).clamp(0.0, 40.0)
    } else {
        0.0
    };

    let blacks = if p001e < 0.005 {
        (50.0 * (1.0 - p001e / 0.005)).clamp(0.0, 50.0)
    } else if p02e > 0.10 {
        (-40.0 * ((p02e - 0.10) / 0.20)).clamp(-40.0, 0.0)
    } else {
        0.0
    };

    let iqr = (p75e - p25e).max(0.0);
    let contrast = if dynamic_range > 0.75 {
        0.0
    } else if iqr < 0.15 {
        (50.0 * ((0.15 - iqr) / 0.15)).clamp(0.0, 50.0)
    } else if iqr > 0.45 {
        (-40.0 * ((iqr - 0.45) / 0.30)).clamp(-40.0, 0.0)
    } else {
        0.0
    };

    let vibrance = if s.high_sat_frac < 0.35 {
        let target = 0.30f32;
        if s.mean_sat < target {
            (90.0 * ((target - s.mean_sat) / target)).clamp(0.0, 60.0)
        } else {
            0.0
        }
    } else {
        0.0
    };

    Edits {
        basic: BasicEdits {
            exposure_ev: exposure_ev as f64,
            contrast: contrast as f64,
            vibrance: vibrance as f64,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: highlights as f64,
            shadows: shadows as f64,
            blacks: blacks as f64,
            whites: whites as f64,
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
            cam_to_xyz: [[0.0; 4]; 3],
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            data,
            cpp: 3,
            orientation: (false, false, false),
            exif: None,
        }
    }

    fn make_frame_gradient(w: usize, h: usize) -> RawFrame {
        let n = w * h;
        let mut data = Vec::with_capacity(n * 3);
        for i in 0..n {
            let v = i as f32 / (n - 1).max(1) as f32;
            data.push(v);
            data.push(v);
            data.push(v);
        }
        RawFrame {
            width: w,
            height: h,
            cfa_pattern: String::new(),
            bps: 16,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            cam_to_xyz: [[0.0; 4]; 3],
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            data,
            cpp: 3,
            orientation: (false, false, false),
            exif: None,
        }
    }

    #[test]
    fn dark_image_gets_positive_exposure() {
        let f = make_frame_luma(0.05, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev <= 0.5 {
            panic!("expected positive exposure, got {}", e.basic.exposure_ev);
        }
    }

    #[test]
    fn bright_image_gets_negative_exposure() {
        let f = make_frame_luma(0.8, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev >= -0.5 {
            panic!("expected negative exposure, got {}", e.basic.exposure_ev);
        }
    }

    #[test]
    fn neutral_image_no_exposure_change() {
        let f = make_frame_luma(0.18, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev.abs() > 0.1 {
            panic!("expected ~0 exposure, got {}", e.basic.exposure_ev);
        }
    }

    #[test]
    fn flat_image_gets_contrast_boost() {
        let f = make_frame_luma(0.18, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.contrast <= 30.0 {
            panic!(
                "expected contrast boost on flat image, got {}",
                e.basic.contrast
            );
        }
    }

    #[test]
    fn high_dynamic_range_no_contrast_boost() {
        let f = make_frame_gradient(256, 256);
        let e = auto_adjust(&f);
        if e.basic.contrast > 0.0 {
            panic!(
                "expected no contrast boost on wide gradient, got {}",
                e.basic.contrast
            );
        }
    }

    #[test]
    fn gray_image_gets_vibrance_boost() {
        let f = make_frame_luma(0.5, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.vibrance <= 30.0 {
            panic!(
                "expected vibrance boost on gray image, got {}",
                e.basic.vibrance
            );
        }
    }

    #[test]
    fn clipped_highlights_get_recovered() {
        let mut data: Vec<f32> = Vec::with_capacity(64 * 64 * 3);
        for i in 0..(64 * 64) {
            let v = if i % 4 == 0 { 0.99 } else { 0.2 };
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
            cam_to_xyz: [[0.0; 4]; 3],
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            data,
            cpp: 3,
            orientation: (false, false, false),
            exif: None,
        };
        let e = auto_adjust(&f);
        if e.tone.highlights >= 0.0 {
            panic!("expected highlight recovery, got {}", e.tone.highlights);
        }
    }
}
