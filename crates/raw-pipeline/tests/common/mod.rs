#![allow(dead_code)]

use raw_pipeline::GpuRenderer;
use raw_pipeline::frame::{OutputFormat, RawFrame, RenderOptions, RenderedImage};
use std::path::{Path, PathBuf};

const RAW_EXTS: &[&str] = &[
    "arw", "cr2", "cr3", "crw", "dng", "erf", "gpr", "iiq", "mrw", "nef", "nrw", "orf", "pef",
    "raf", "raw", "rw2", "rwl", "sr2", "srw", "x3f",
];

pub fn any_fixture() -> Option<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let entries = std::fs::read_dir(&dir).ok()?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| RAW_EXTS.contains(&e.to_ascii_lowercase().as_str()))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();
    paths.into_iter().next()
}

pub fn try_renderer() -> Option<GpuRenderer> {
    match GpuRenderer::new() {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            None
        }
    }
}

pub fn rgb_frame(w: usize, h: usize, data: Vec<f32>) -> RawFrame {
    RawFrame {
        width: w,
        height: h,
        cfa_pattern: String::new(),
        bps: 16,
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        xyz_to_cam: [[0.0; 3]; 4],
        color_matrices: Vec::new(),
        data,
        cpp: 3,
        orientation: (false, false, false),
        is_raw: false,
        capture_sigma: None,
        model: String::new(),
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

pub fn fine_texture_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    let period = 32.0f32;
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let u = x as f32 / (w - 1) as f32;
            let v = y as f32 / (h - 1) as f32;
            let base = 0.2 + 0.5 * u * (1.0 - 0.4 * v);
            let ripple = (std::f32::consts::TAU * x as f32 / period).sin()
                * (std::f32::consts::TAU * y as f32 / period).sin();
            for c in 0..3 {
                let amp = 0.06 * (1.0 - 0.2 * c as f32);
                data[i + c] = (base * (1.0 - 0.06 * c as f32) + amp * ripple).clamp(0.02, 0.95);
            }
        }
    }
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
        width: w,
        height: h,
        cfa_pattern: cfa_pattern.to_string(),
        bps: 16,
        wb_coeffs: [1.0, 1.0, 1.0, 1.0],
        xyz_to_cam: [[0.0; 3]; 4],
        color_matrices: Vec::new(),
        data,
        cpp: 1,
        orientation: (false, false, false),
        is_raw: false,
        capture_sigma: None,
        model: String::new(),
        exif: None,
    }
}

pub fn rgb8_opts(max_edge: u32) -> RenderOptions {
    RenderOptions {
        max_edge,
        output: OutputFormat::Rgb8,
        ..Default::default()
    }
}

pub fn report_mode() -> bool {
    std::env::var("PARITY_REPORT").is_ok_and(|v| v == "1")
}

pub fn require_same_dims(label: &str, cpu: &RenderedImage, gpu: &RenderedImage) {
    if cpu.width != gpu.width || cpu.height != gpu.height {
        panic!(
            "{label}: dim mismatch CPU {}x{} vs GPU {}x{}",
            cpu.width, cpu.height, gpu.width, gpu.height
        );
    }
}

pub struct ParityLedger {
    what: &'static str,
    rows: Vec<(String, f64, f64)>,
}

impl ParityLedger {
    pub fn new(what: &'static str) -> Self {
        Self {
            what,
            rows: Vec::new(),
        }
    }

    pub fn check(&mut self, label: &str, cpu: &[u8], gpu: &[u8], limit: f64) -> f64 {
        let delta = mean_abs_delta(cpu, gpu);
        eprintln!(
            "PARITY {}/{label} delta={delta:.4} limit={limit}",
            self.what
        );
        self.rows.push((label.to_string(), delta, limit));
        delta
    }

    pub fn finish(self) {
        let failed: Vec<String> = self
            .rows
            .iter()
            .filter(|(_, delta, limit)| delta > limit)
            .map(|(label, delta, limit)| format!("{label}: {delta:.4} > {limit}"))
            .collect();
        if failed.is_empty() {
            return;
        }
        if report_mode() {
            eprintln!("PARITY {} over limit: {}", self.what, failed.join("; "));
            return;
        }
        panic!("{} CPU/GPU drift: {}", self.what, failed.join("; "));
    }
}

pub fn mean_abs_delta(a: &[u8], b: &[u8]) -> f64 {
    if a.len() != b.len() {
        panic!("len mismatch: {} vs {}", a.len(), b.len());
    }
    let sum: u64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x as i32 - y as i32).unsigned_abs() as u64)
        .sum();
    sum as f64 / a.len() as f64
}

pub fn mean_abs_laplacian(rgb: &[u8], w: usize, h: usize) -> f64 {
    if w < 3 || h < 3 {
        panic!("image too small for laplacian: {w}x{h}");
    }
    let luma = |x: usize, y: usize| {
        let i = (y * w + x) * 3;
        0.2126 * rgb[i] as f64 + 0.7152 * rgb[i + 1] as f64 + 0.0722 * rgb[i + 2] as f64
    };
    let sum: f64 = (1..h - 1)
        .flat_map(|y| (1..w - 1).map(move |x| (x, y)))
        .map(|(x, y)| {
            (4.0 * luma(x, y) - luma(x - 1, y) - luma(x + 1, y) - luma(x, y - 1) - luma(x, y + 1))
                .abs()
        })
        .sum();
    sum / ((w - 2) * (h - 2)) as f64
}

pub fn warn_pixels(rgb: &[u8], color: [u8; 3]) -> Vec<usize> {
    rgb.chunks_exact(3)
        .enumerate()
        .filter(|(_, p)| p[0] == color[0] && p[1] == color[1] && p[2] == color[2])
        .map(|(i, _)| i)
        .collect()
}
