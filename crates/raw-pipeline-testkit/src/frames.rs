use raw_pipeline::frame::{FrameMeta, RawFrame};

pub fn rgb_frame(w: usize, h: usize, data: Vec<f32>) -> RawFrame {
    RawFrame {
        meta: FrameMeta {
            width: w,
            height: h,
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
    }
}

pub fn synthetic_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / (w - 1) as f32;
            let v = y as f32 / (h - 1) as f32;
            let i = (y * w + x) * 3;
            data[i] = (u * 1.2).clamp(0.0, 1.5);
            data[i + 1] = (v * 1.0).clamp(0.0, 1.5);
            data[i + 2] = ((u + v) * 0.5 * 1.1).clamp(0.0, 1.5);
        }
    }
    rgb_frame(w, h, data)
}

pub fn ramp_frame(w: usize, h: usize) -> RawFrame {
    let data = (0..h)
        .flat_map(|_| 0..w)
        .flat_map(|x| {
            let u = x as f32 / (w - 1) as f32;
            [u, u, u]
        })
        .collect();
    rgb_frame(w, h, data)
}

pub fn detail_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let checker = if (x / 2 + y / 2) % 2 == 0 {
                0.16
            } else {
                -0.16
            };
            let base = 0.45 + 0.2 * (x as f32 / (w - 1) as f32);
            data[i] = (base + checker).clamp(0.02, 0.98);
            data[i + 1] = (base * 0.95 + checker).clamp(0.02, 0.98);
            data[i + 2] = (base * 0.9 - checker).clamp(0.02, 0.98);
        }
    }
    rgb_frame(w, h, data)
}

pub fn step_edge_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let step = if x < w / 2 { 0.18 } else { 0.62 };
            let ripple = 0.05
                * (std::f32::consts::TAU * x as f32 / 7.0).sin()
                * (std::f32::consts::TAU * y as f32 / 5.0).sin();
            let level = step + ripple;
            data[i] = level;
            data[i + 1] = level * 0.9;
            data[i + 2] = level * 0.8;
        }
    }
    rgb_frame(w, h, data)
}

pub fn noisy_frame(w: usize, h: usize) -> RawFrame {
    let mut seed: u32 = 0x9e37_79b9;
    let mut gaussian = || {
        (0..12)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as f32 / u32::MAX as f32
            })
            .sum::<f32>()
            - 6.0
    };
    let data = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            let u = x as f32 / (w - 1) as f32;
            let v = y as f32 / (h - 1) as f32;
            let step = if x < w / 2 { 0.08 } else { 0.35 };
            [
                step + 0.2 * u,
                step * 0.9 + 0.15 * v,
                step * 0.8 + 0.1 * u * v,
            ]
        })
        .map(|signal: f32| (signal + (0.0004 + 0.004 * signal).sqrt() * gaussian()).clamp(0.0, 1.0))
        .collect();
    rgb_frame(w, h, data)
}

pub fn haze_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let fx = x as f32 / w as f32;
            let fy = y as f32 / h as f32;
            let base = 0.15 + 0.6 * fx;
            let haze = 0.45 * (1.0 - fy);
            data[i] = (base + haze).min(1.0);
            data[i + 1] = (base * 0.9 + haze).min(1.0);
            data[i + 2] = (base * 0.8 + haze * 1.1).min(1.0);
        }
    }
    rgb_frame(w, h, data)
}

pub fn stripe_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let stripe = ((x / 4) % 2) as f32;
            data[i] = 0.2 + 0.5 * stripe;
            data[i + 1] = 0.2 + 0.5 * stripe;
            data[i + 2] = 0.2 + 0.5 * stripe;
        }
    }
    rgb_frame(w, h, data)
}

pub fn split_tone_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let dark = if x < w / 2 { 0.05 } else { 0.6 };
            data[i] = dark;
            data[i + 1] = dark;
            data[i + 2] = dark;
        }
    }
    rgb_frame(w, h, data)
}

pub fn synthetic_bayer_frame(w: usize, h: usize, cfa_pattern: &str) -> RawFrame {
    let mut data = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let block = (x / 4 + y / 4) % 2;
            data[y * w + x] = if block == 0 { 0.2 } else { 0.8 };
        }
    }
    RawFrame {
        cfa_pattern: cfa_pattern.to_string(),
        cpp: 1,
        ..rgb_frame(w, h, data)
    }
}
