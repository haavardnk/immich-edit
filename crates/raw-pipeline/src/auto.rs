use crate::edits::Edits;
use crate::frame::RawFrame;

const SAMPLE_TARGET: usize = 200_000;

fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn auto_adjust(frame: &RawFrame) -> Edits {
    let total = frame.width * frame.height;
    if total == 0 {
        return Edits::default();
    }
    let step = (total / SAMPLE_TARGET).max(1);

    let wb = frame.wb_coeffs;
    let max_wb = wb[0].max(wb[1]).max(wb[2]).max(1e-6);
    let scale = [wb[0] / max_wb, wb[1] / max_wb, wb[2] / max_wb];

    let mut lumas: Vec<f32> = Vec::with_capacity(total / step + 1);
    let mut i = 0;
    while i < total {
        let off = i * 3;
        let r = frame.data[off] * scale[0];
        let g = frame.data[off + 1] * scale[1];
        let b = frame.data[off + 2] * scale[2];
        let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        lumas.push(y.clamp(0.0, 4.0));
        i += step;
    }

    lumas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let p1 = percentile(&lumas, 0.01);
    let p50 = percentile(&lumas, 0.50);
    let p99 = percentile(&lumas, 0.99);

    let target_median: f32 = 0.18;
    let exposure_ev = if p50 > 1e-4 {
        (target_median / p50).log2().clamp(-2.0, 2.0)
    } else {
        2.0
    };

    let shadow_floor = 0.05;
    let shadows = if p1 < shadow_floor {
        (30.0 * (1.0 - p1 / shadow_floor)).clamp(0.0, 40.0)
    } else {
        0.0
    };

    let highlight_ceil = 0.95;
    let highlights = if p99 > highlight_ceil {
        (-30.0 * ((p99 - highlight_ceil) / (1.0 - highlight_ceil))).clamp(-40.0, 0.0)
    } else {
        0.0
    };

    Edits {
        basic: crate::edits::BasicEdits {
            exposure_ev: exposure_ev as f64,
            ..Default::default()
        },
        tone: crate::edits::ToneEdits {
            highlights: highlights as f64,
            shadows: shadows as f64,
        },
        geometry: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(luma: f32, w: usize, h: usize) -> RawFrame {
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

    #[test]
    fn dark_image_gets_positive_exposure() {
        let f = make_frame(0.05, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev <= 0.5 {
            panic!("expected positive exposure, got {}", e.basic.exposure_ev);
        }
    }

    #[test]
    fn bright_image_gets_negative_exposure() {
        let f = make_frame(0.8, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev >= -0.5 {
            panic!("expected negative exposure, got {}", e.basic.exposure_ev);
        }
    }

    #[test]
    fn neutral_image_no_exposure_change() {
        let f = make_frame(0.18, 64, 64);
        let e = auto_adjust(&f);
        if e.basic.exposure_ev.abs() > 0.1 {
            panic!("expected ~0 exposure, got {}", e.basic.exposure_ev);
        }
    }
}
