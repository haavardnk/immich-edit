use raw_pipeline::edits::BasicEdits;
use raw_pipeline::edits::{
    CropRect, CurvePoint, CurvePoints, CurvesEdits, DetailEdits, EffectsEdits, GeometryEdits,
    LensEdits,
};
use raw_pipeline::frame::{BitDepth, OutputFormat, PngCompression, RawFrame};
use raw_pipeline::{GpuRenderer, decode, edits::Edits, frame::RenderOptions};
use std::path::{Path, PathBuf};

const RAW_EXTS: &[&str] = &[
    "arw", "cr2", "cr3", "crw", "dng", "erf", "gpr", "iiq", "mrw", "nef", "nrw", "orf", "pef",
    "raf", "raw", "rw2", "rwl", "sr2", "srw", "x3f",
];

fn any_fixture() -> Option<PathBuf> {
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

fn try_renderer() -> Option<GpuRenderer> {
    match GpuRenderer::new() {
        Ok(r) => Some(r),
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            None
        }
    }
}

#[test]
fn gpu_identity_render_jpeg() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        eprintln!("no fixture, skipping");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: 512,
        ..Default::default()
    };
    let out = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    if out.bytes.len() < 1000 {
        panic!("jpeg too small ({} bytes)", out.bytes.len());
    }
    if &out.bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg SOI marker");
    }
    if out.width.max(out.height) > 512 {
        panic!("max edge exceeded {}x{}", out.width, out.height);
    }
    if out.renderer != "gpu" {
        panic!("renderer label: {}", out.renderer);
    }
}

#[test]
fn gpu_exposure_brightens() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: 256,
        ..Default::default()
    };

    let base = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let bright = Edits {
        basic: raw_pipeline::edits::BasicEdits {
            exposure_ev: 2.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let bumped = renderer.render(&frame, &bright, &opts).unwrap();

    let mean_base: f64 = base
        .histogram
        .l
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / base.histogram.l.iter().sum::<u32>().max(1) as f64;
    let mean_bumped: f64 = bumped
        .histogram
        .l
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / bumped.histogram.l.iter().sum::<u32>().max(1) as f64;

    if mean_bumped <= mean_base {
        panic!("exposure did not brighten: {mean_base} -> {mean_bumped}");
    }
}

#[test]
fn gpu_rotate_swaps_dims() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: 512,
        ..Default::default()
    };

    let a = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let rotated = Edits {
        geometry: raw_pipeline::edits::GeometryEdits {
            rotate: 90,
            ..Default::default()
        },
        ..Default::default()
    };
    let b = renderer.render(&frame, &rotated, &opts).unwrap();

    let landscape_a = a.width >= a.height;
    let landscape_b = b.width >= b.height;
    if landscape_a == landscape_b {
        panic!(
            "rotate did not swap orientation: {}x{} -> {}x{}",
            a.width, a.height, b.width, b.height
        );
    }
}

fn synthetic_frame(w: usize, h: usize) -> RawFrame {
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

fn detail_frame(w: usize, h: usize) -> RawFrame {
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

fn fine_texture_frame(w: usize, h: usize) -> RawFrame {
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

fn haze_frame(w: usize, h: usize) -> RawFrame {
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

fn decode_jpeg_rgb(jpeg: &[u8]) -> (Vec<u8>, usize, usize) {
    let img: turbojpeg::Image<Vec<u8>> =
        turbojpeg::decompress(jpeg, turbojpeg::PixelFormat::RGB).unwrap();
    (img.pixels, img.width, img.height)
}

fn mean_abs_delta(a: &[u8], b: &[u8]) -> f64 {
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

#[test]
fn gpu_matches_cpu_within_tolerance() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };

    let cases: &[(&str, f64, Edits)] = &[
        ("identity", 1.0, Edits::default()),
        (
            "exposure+1.5",
            1.1,
            Edits {
                basic: BasicEdits {
                    exposure_ev: 1.5,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "saturation+50",
            1.0,
            Edits {
                basic: BasicEdits {
                    saturation: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "contrast+30",
            2.0,
            Edits {
                basic: BasicEdits {
                    contrast: 30.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "brightness+35",
            2.0,
            Edits {
                basic: BasicEdits {
                    brightness: 35.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "whites+50",
            1.5,
            Edits {
                tone: raw_pipeline::edits::ToneEdits {
                    whites: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "blacks+50",
            1.5,
            Edits {
                tone: raw_pipeline::edits::ToneEdits {
                    blacks: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "highlights-100",
            2.5,
            Edits {
                tone: raw_pipeline::edits::ToneEdits {
                    highlights: -100.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "rotate10+crop",
            2.5,
            Edits {
                geometry: GeometryEdits {
                    rotate_angle: 10.0,
                    crop: Some(CropRect {
                        x: 0.15,
                        y: 0.15,
                        w: 0.7,
                        h: 0.7,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "perspective+rotate+crop",
            2.5,
            Edits {
                geometry: GeometryEdits {
                    rotate_angle: 6.0,
                    crop: Some(CropRect {
                        x: 0.12,
                        y: 0.12,
                        w: 0.7,
                        h: 0.7,
                    }),
                    perspective: Some(raw_pipeline::perspective::PerspectiveEdits {
                        vertical: 45.0,
                        horizontal: -20.0,
                        aspect: 15.0,
                        corners: None,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "per_channel_curves",
            2.0,
            Edits {
                basic: BasicEdits {
                    curves: CurvesEdits {
                        composite: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.04 },
                                CurvePoint { x: 0.5, y: 0.55 },
                                CurvePoint { x: 1.0, y: 0.97 },
                            ],
                        },
                        r: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.0 },
                                CurvePoint { x: 0.5, y: 0.62 },
                                CurvePoint { x: 1.0, y: 1.0 },
                            ],
                        },
                        g: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.0 },
                                CurvePoint { x: 0.5, y: 0.42 },
                                CurvePoint { x: 1.0, y: 1.0 },
                            ],
                        },
                        b: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.0 },
                                CurvePoint { x: 0.5, y: 0.58 },
                                CurvePoint { x: 1.0, y: 1.0 },
                            ],
                        },
                        luma: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.02 },
                                CurvePoint { x: 0.4, y: 0.45 },
                                CurvePoint { x: 1.0, y: 0.98 },
                            ],
                        },
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_distortion_barrel",
            4.0,
            Edits {
                lens: LensEdits {
                    profile_enabled: true,
                    distortion_amount: 100.0,
                    k1: -0.1,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_vignette_brighten",
            3.0,
            Edits {
                lens: LensEdits {
                    profile_enabled: true,
                    vignette_amount: 100.0,
                    vk1: -0.4,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_ca_red",
            4.0,
            Edits {
                lens: LensEdits {
                    ca_enabled: true,
                    ca_red_scale_x10000: 80.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_combo",
            5.0,
            Edits {
                lens: LensEdits {
                    profile_enabled: true,
                    ca_enabled: true,
                    distortion_amount: 100.0,
                    k1: -0.08,
                    vignette_amount: 100.0,
                    vk1: -0.3,
                    ca_red_scale_x10000: 50.0,
                    ca_blue_scale_x10000: -50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
    ];

    let mut failed: Vec<String> = Vec::new();

    for (label, threshold, edits) in cases {
        let cpu_out = raw_pipeline::cpu::render(&frame, edits, &opts).unwrap();
        let gpu_out = renderer.render(&frame, edits, &opts).unwrap();

        if cpu_out.width != gpu_out.width || cpu_out.height != gpu_out.height {
            panic!(
                "{label}: dim mismatch CPU {}x{} vs GPU {}x{}",
                cpu_out.width, cpu_out.height, gpu_out.width, gpu_out.height
            );
        }
        let (cpu_rgb, cw, ch) = decode_jpeg_rgb(&cpu_out.bytes);
        let (gpu_rgb, gw, gh) = decode_jpeg_rgb(&gpu_out.bytes);
        if (cw, ch) != (gw, gh) {
            panic!("{label}: decoded dim mismatch {cw}x{ch} vs {gw}x{gh}");
        }
        let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
        eprintln!("{label}: mean abs delta = {delta:.3}");
        if delta > *threshold {
            failed.push(format!("{label}: {delta:.3} > {threshold}"));
        }
    }
    if !failed.is_empty() {
        panic!("CPU vs GPU drift exceeded threshold: {}", failed.join("; "));
    }
}

#[test]
fn gpu_exif_orientation_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 256,
        ..Default::default()
    };
    let w: usize = 40;
    let h: usize = 30;
    let data = vec![0.5f32; w * h * 3];

    let orientations: &[((bool, bool, bool), &str)] = &[
        ((false, false, false), "Normal"),
        ((false, true, false), "HorizontalFlip"),
        ((false, false, true), "VerticalFlip"),
        ((false, true, true), "Rotate180"),
        ((true, false, false), "Transpose"),
        ((true, false, true), "Rotate90"),
        ((true, true, false), "Rotate270"),
        ((true, true, true), "Transverse"),
    ];

    for &(orient, label) in orientations {
        let frame = RawFrame {
            width: w,
            height: h,
            cfa_pattern: String::new(),
            bps: 16,
            wb_coeffs: [1.0, 1.0, 1.0, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            data: data.clone(),
            cpp: 3,
            orientation: orient,
            is_raw: false,
            capture_sigma: None,
            model: String::new(),
            exif: None,
        };

        let gpu_out = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let cpu_out = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();

        if gpu_out.width != cpu_out.width || gpu_out.height != cpu_out.height {
            panic!(
                "{label}: GPU {}x{} != CPU {}x{}",
                gpu_out.width, gpu_out.height, cpu_out.width, cpu_out.height
            );
        }
    }
}

#[test]
fn gpu_presence_sliders_match_cpu_via_fallback() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let w: usize = 48;
    let h: usize = 32;
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
    let frame = RawFrame {
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
    };
    let mut edits = Edits::default();
    edits.basic.texture = 30.0;
    edits.basic.clarity = 20.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("presence mean abs delta = {delta:.3}");
    if delta > 8.0 {
        panic!("presence GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_dehaze_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let frame = haze_frame(96, 64);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("dehaze mean abs delta = {delta:.3}");
    if delta > 10.0 {
        panic!("dehaze GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_dehaze_with_presence_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let frame = haze_frame(96, 64);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;
    edits.basic.clarity = 80.0;
    edits.basic.texture = 80.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("dehaze+presence mean abs delta = {delta:.3}");
    if delta > 1.2 {
        panic!("dehaze+presence GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_shadows_match_cpu_via_pyramid() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let w: usize = 48;
    let h: usize = 32;
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
    let frame = RawFrame {
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
    };
    let mut edits = Edits::default();
    edits.tone.shadows = 50.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("shadows mean abs delta = {delta:.3}");
    if delta > 8.0 {
        panic!("shadows GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: 80.0,
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("sharpen mean abs delta = {delta:.3}");
    if delta > 3.0 {
        panic!("sharpen GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_sharpen_masking_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: 80.0,
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 60.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("sharpen+masking mean abs delta = {delta:.3}");
    if delta > 3.0 {
        panic!("sharpen+masking GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_nr_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            luma_nr_amount: 50.0,
            luma_nr_detail: 40.0,
            luma_nr_contrast: 0.0,
            color_nr_amount: 50.0,
            color_nr_detail: 40.0,
            color_nr_smoothness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("nr mean abs delta = {delta:.3}");
    if delta > 4.0 {
        panic!("nr GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_nr_matches_cpu_with_preview_downsample() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 320,
        quality: false,
        ..Default::default()
    };
    let frame = fine_texture_frame(1024, 768);
    let edits = Edits {
        detail: DetailEdits {
            luma_nr_amount: 100.0,
            luma_nr_detail: 30.0,
            luma_nr_contrast: 0.0,
            color_nr_amount: 60.0,
            color_nr_detail: 40.0,
            color_nr_smoothness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("nr preview-downsample mean abs delta = {delta:.3}");
    if delta > 1.5 {
        panic!("nr preview GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_capture_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 256,
        quality: true,
        ..Default::default()
    };
    let mut frame = detail_frame(256, 192);
    frame.is_raw = true;
    frame.capture_sigma = Some(0.7);
    let edits = Edits::default();
    let off = Edits {
        detail: DetailEdits {
            capture_sharpen: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_off = renderer.render(&frame, &off, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let (off_rgb, _, _) = decode_jpeg_rgb(&gpu_off.bytes);
    let effect = mean_abs_delta(&gpu_rgb, &off_rgb);
    eprintln!("capture sharpen effect = {effect:.3}");
    if effect < 0.5 {
        panic!("capture sharpen had no visible effect: {effect:.3}");
    }
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("capture sharpen mean abs delta = {delta:.3}");
    if delta > 1.0 {
        panic!("capture sharpen GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_vignette_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        effects: EffectsEdits {
            vignette_amount: -50.0,
            vignette_midpoint: 40.0,
            vignette_feather: 60.0,
            vignette_roundness: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("vignette mean abs delta = {delta:.3}");
    if delta > 2.0 {
        panic!("vignette GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_grain_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        effects: EffectsEdits {
            grain_amount: 50.0,
            grain_size: 25.0,
            grain_roughness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("grain mean abs delta = {delta:.3}");
    if delta > 6.0 {
        panic!("grain GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_effects_with_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: 60.0,
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        effects: EffectsEdits {
            vignette_amount: -40.0,
            grain_amount: 30.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("sharpen+effects mean abs delta = {delta:.3}");
    if delta > 6.0 {
        panic!("sharpen+effects GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_linear_histogram_changes_with_vignette() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let neutral = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let edited = renderer
        .render(
            &frame,
            &Edits {
                effects: EffectsEdits {
                    vignette_amount: -80.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            &opts,
        )
        .unwrap();
    let nh = neutral.linear_histogram.expect("neutral linear histogram");
    let eh = edited.linear_histogram.expect("edited linear histogram");
    let diff: i64 =
        nh.l.iter()
            .zip(eh.l.iter())
            .map(|(a, b)| (*a as i64 - *b as i64).abs())
            .sum();
    if diff == 0 {
        panic!("linear histogram unchanged after vignette");
    }
}

fn synthetic_bayer_frame(w: usize, h: usize, cfa_pattern: &str) -> RawFrame {
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

fn decode_png_rgb(bytes: &[u8]) -> (Vec<u8>, usize, usize) {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0u8; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();
    buf.truncate(info.buffer_size());
    (buf, info.width as usize, info.height as usize)
}

#[test]
fn gpu_demosaic_matches_cpu_mhc() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };

    let patterns: &[(&str, usize, usize)] = &[
        ("RGGB", 96, 64),
        ("BGGR", 88, 60),
        ("GRBG", 80, 56),
        ("GBRG", 72, 52),
    ];

    let mut failed: Vec<String> = Vec::new();
    for (cfa, w, h) in patterns {
        let frame = synthetic_bayer_frame(*w, *h, cfa);
        let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
        let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        if cpu.width != gpu.width || cpu.height != gpu.height {
            panic!(
                "{cfa}: dim mismatch CPU {}x{} vs GPU {}x{}",
                cpu.width, cpu.height, gpu.width, gpu.height
            );
        }
        let (cpu_rgb, cw, ch) = decode_png_rgb(&cpu.bytes);
        let (gpu_rgb, gw, gh) = decode_png_rgb(&gpu.bytes);
        if (cw, ch) != (gw, gh) {
            panic!("{cfa}: decoded dim mismatch {cw}x{ch} vs {gw}x{gh}");
        }
        let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
        eprintln!("demosaic {cfa}: mean abs delta = {delta:.3}");
        if delta > 1.0 {
            failed.push(format!("{cfa}: {delta:.3} > 1.0"));
        }
    }
    if !failed.is_empty() {
        panic!("GPU demosaic drift: {}", failed.join("; "));
    }
}

#[test]
fn gpu_xtrans_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };

    const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";
    let sizes: &[(usize, usize)] = &[(96, 66), (86, 58)];

    let mut failed: Vec<String> = Vec::new();
    for (w, h) in sizes {
        let frame = synthetic_bayer_frame(*w, *h, XTRANS);
        let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
        let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        if cpu.width != gpu.width || cpu.height != gpu.height {
            panic!(
                "{w}x{h}: dim mismatch CPU {}x{} vs GPU {}x{}",
                cpu.width, cpu.height, gpu.width, gpu.height
            );
        }
        let (cpu_rgb, _, _) = decode_png_rgb(&cpu.bytes);
        let (gpu_rgb, _, _) = decode_png_rgb(&gpu.bytes);
        let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
        eprintln!("xtrans {w}x{h}: mean abs delta = {delta:.3}");
        if delta > 1.0 {
            failed.push(format!("{w}x{h}: {delta:.3} > 1.0"));
        }
    }
    if !failed.is_empty() {
        panic!("GPU X-Trans demosaic drift: {}", failed.join("; "));
    }
}

#[test]
fn gpu_xtrans_fixture_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("no X-Trans fixture; skipping");
        return;
    };
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: 4096,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Eight,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    if cpu.width != gpu.width || cpu.height != gpu.height {
        panic!(
            "dim mismatch CPU {}x{} vs GPU {}x{}",
            cpu.width, cpu.height, gpu.width, gpu.height
        );
    }
    let (cpu_rgb, _, _) = decode_png_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_png_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("xtrans fixture mean abs delta = {delta:.3}");
    if delta > 2.0 {
        panic!("GPU X-Trans fixture drift too high: {delta:.3}");
    }
}

#[test]
fn gpu_masks_match_cpu_within_tolerance() {
    use raw_pipeline::edits::{
        MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
        Vec2f,
    };

    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };

    for invert in [false, true] {
        let layer = MaskLayer {
            id: "L1".into(),
            name: "".into(),
            enabled: true,
            color: "#ff3b30".into(),
            amount: 1.0,
            invert,
            components: vec![
                MaskComponent {
                    id: "c1".into(),
                    enabled: true,
                    mode: MaskComponentMode::Add,
                    invert: false,
                    kind: MaskComponentKind::Linear {
                        p0: Vec2f { x: 0.0, y: 0.5 },
                        p1: Vec2f { x: 1.0, y: 0.5 },
                        feather: 0.4,
                    },
                    source: MaskSource::Manual,
                    generated: None,
                },
                MaskComponent {
                    id: "c2".into(),
                    enabled: true,
                    mode: MaskComponentMode::Subtract,
                    invert: false,
                    kind: MaskComponentKind::Radial {
                        center: Vec2f { x: 0.25, y: 0.5 },
                        radius_xy: Vec2f { x: 0.2, y: 0.2 },
                        feather: 0.3,
                    },
                    source: MaskSource::Manual,
                    generated: None,
                },
            ],
            edits: MaskedEdits {
                exposure_ev: Some(1.2),
                brightness: Some(25.0),
                saturation: Some(30.0),
                contrast: Some(20.0),
                wb_temp: Some(15.0),
                ..Default::default()
            },
        };

        let edits = Edits {
            masks: vec![layer],
            ..Default::default()
        };

        let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
        if cpu_out.width != gpu_out.width || cpu_out.height != gpu_out.height {
            panic!(
                "dim mismatch CPU {}x{} vs GPU {}x{}",
                cpu_out.width, cpu_out.height, gpu_out.width, gpu_out.height
            );
        }
        let (cpu_rgb, cw, ch) = decode_jpeg_rgb(&cpu_out.bytes);
        let (gpu_rgb, gw, gh) = decode_jpeg_rgb(&gpu_out.bytes);
        if (cw, ch) != (gw, gh) {
            panic!("decoded dim mismatch {cw}x{ch} vs {gw}x{gh}");
        }
        let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
        eprintln!("masks parity (invert={invert}): mean abs delta = {delta:.3}");
        if delta > 1.5 {
            panic!("CPU vs GPU mask drift with invert={invert}: {delta:.3} > 1.5");
        }
    }
}

#[test]
fn gpu_brush_masks_match_cpu_within_tolerance() {
    use raw_pipeline::edits::{
        MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    };
    use raw_pipeline::mask_raster::{MaskRaster, RasterMap};
    use std::sync::Arc;

    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let w: u32 = 32;
    let h: u32 = 32;
    let mut bytes = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if x >= w / 2 {
                bytes[(y * w + x) as usize] = 255;
            }
        }
    }
    let raster = Arc::new(MaskRaster::new(w, h, bytes).unwrap());
    let mut rasters = RasterMap::new();
    rasters.insert("brush_a".into(), raster);

    let opts = RenderOptions {
        max_edge: 96,
        rasters,
        ..Default::default()
    };

    let layer = MaskLayer {
        id: "L1".into(),
        name: "".into(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "b1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "brush_a".into(),
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            exposure_ev: Some(1.5),
            saturation: Some(25.0),
            ..Default::default()
        },
    };

    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };

    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    if cpu_out.width != gpu_out.width || cpu_out.height != gpu_out.height {
        panic!(
            "dim mismatch CPU {}x{} vs GPU {}x{}",
            cpu_out.width, cpu_out.height, gpu_out.width, gpu_out.height
        );
    }
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu_out.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu_out.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("brush masks parity: mean abs delta = {delta:.3}");
    if delta > 2.0 {
        panic!("CPU vs GPU brush mask drift: {delta:.3} > 2.0");
    }
}

#[test]
fn gpu_masked_presence_matches_cpu_and_changes_output() {
    use raw_pipeline::edits::{
        MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
        Vec2f,
    };

    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let layer = MaskLayer {
        id: "L1".into(),
        name: String::new(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Linear {
                p0: Vec2f { x: 0.0, y: 0.5 },
                p1: Vec2f { x: 1.0, y: 0.5 },
                feather: 0.2,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            texture: Some(80.0),
            clarity: Some(60.0),
            ..Default::default()
        },
    };
    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };
    let plain = Edits::default();

    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu_out.bytes);
    let (cpu_plain_rgb, _, _) = decode_jpeg_rgb(&cpu_plain.bytes);
    let cpu_effect = mean_abs_delta(&cpu_rgb, &cpu_plain_rgb);
    eprintln!("masked presence cpu effect = {cpu_effect:.3}");
    if cpu_effect < 0.5 {
        panic!("masked texture and clarity had no effect on the CPU path: {cpu_effect:.3}");
    }

    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu_out.bytes);
    let (gpu_plain_rgb, _, _) = decode_jpeg_rgb(&gpu_plain.bytes);
    let gpu_effect = mean_abs_delta(&gpu_rgb, &gpu_plain_rgb);
    eprintln!("masked presence gpu effect = {gpu_effect:.3}");
    if gpu_effect < 0.5 {
        panic!("masked texture and clarity had no effect on the GPU path: {gpu_effect:.3}");
    }

    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("masked presence parity = {delta:.3}");
    if delta > 8.0 {
        panic!("CPU vs GPU masked presence drift: {delta:.3} > 8.0");
    }
}

#[test]
fn gpu_masked_sharpen_matches_cpu_and_changes_output() {
    use raw_pipeline::edits::{
        MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
        Vec2f,
    };

    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = detail_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let layer = MaskLayer {
        id: "L1".into(),
        name: String::new(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Linear {
                p0: Vec2f { x: 0.0, y: 0.5 },
                p1: Vec2f { x: 1.0, y: 0.5 },
                feather: 0.2,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            sharpen: Some(120.0),
            ..Default::default()
        },
    };
    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };
    let plain = Edits::default();

    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
    let (cpu_rgb, w, h) = decode_jpeg_rgb(&cpu_out.bytes);
    let (cpu_plain_rgb, _, _) = decode_jpeg_rgb(&cpu_plain.bytes);
    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu_out.bytes);
    let (gpu_plain_rgb, _, _) = decode_jpeg_rgb(&gpu_plain.bytes);

    let region_delta = |a: &[u8], b: &[u8], from: usize, to: usize| -> f64 {
        let samples: Vec<usize> = (0..h)
            .flat_map(|y| (from..to).flat_map(move |x| (0..3).map(move |c| (y * w + x) * 3 + c)))
            .collect();
        let sum: f64 = samples
            .iter()
            .map(|&i| (a[i] as f64 - b[i] as f64).abs())
            .sum();
        sum / samples.len() as f64
    };

    let cpu_masked = region_delta(&cpu_rgb, &cpu_plain_rgb, w * 3 / 4, w);
    let cpu_clear = region_delta(&cpu_rgb, &cpu_plain_rgb, 0, w / 8);
    eprintln!("masked sharpen cpu masked = {cpu_masked:.3} clear = {cpu_clear:.3}");
    if cpu_masked < 0.5 {
        panic!("masked sharpen had no effect on the CPU path: {cpu_masked:.3}");
    }
    if cpu_clear > 0.2 {
        panic!("masked sharpen leaked outside the mask on the CPU path: {cpu_clear:.3}");
    }

    let gpu_masked = region_delta(&gpu_rgb, &gpu_plain_rgb, w * 3 / 4, w);
    let gpu_clear = region_delta(&gpu_rgb, &gpu_plain_rgb, 0, w / 8);
    eprintln!("masked sharpen gpu masked = {gpu_masked:.3} clear = {gpu_clear:.3}");
    if gpu_masked < 0.5 {
        panic!("masked sharpen had no effect on the GPU path: {gpu_masked:.3}");
    }
    if gpu_clear > 0.2 {
        panic!("masked sharpen leaked outside the mask on the GPU path: {gpu_clear:.3}");
    }

    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("masked sharpen parity = {delta:.3}");
    if delta > 3.0 {
        panic!("CPU vs GPU masked sharpen drift: {delta:.3} > 3.0");
    }
}

#[test]
fn gpu_range_masks_match_cpu_within_tolerance() {
    use raw_pipeline::edits::{
        MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    };

    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    {
        let layer = MaskLayer {
            id: "L1".into(),
            name: String::new(),
            enabled: true,
            color: "#ff3b30".into(),
            amount: 1.0,
            invert: false,
            components: vec![
                MaskComponent {
                    id: "color".into(),
                    enabled: true,
                    mode: MaskComponentMode::Add,
                    invert: false,
                    kind: MaskComponentKind::ColorRange {
                        sample_rgb: [0.65, 0.45, 0.35],
                        tolerance: 0.25,
                        softness: 0.15,
                    },
                    source: MaskSource::Manual,
                    generated: None,
                },
                MaskComponent {
                    id: "luma".into(),
                    enabled: true,
                    mode: MaskComponentMode::Intersect,
                    invert: false,
                    kind: MaskComponentKind::LumaRange {
                        min: 0.15,
                        max: 0.85,
                        softness: 0.15,
                    },
                    source: MaskSource::Manual,
                    generated: None,
                },
            ],
            edits: MaskedEdits {
                exposure_ev: Some(0.8),
                saturation: Some(20.0),
                ..Default::default()
            },
        };
        let edits = Edits {
            masks: vec![layer],
            ..Default::default()
        };
        let opts = RenderOptions {
            max_edge: 96,
            ..Default::default()
        };
        let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
        let (cpu_rgb, cw, ch) = decode_jpeg_rgb(&cpu_out.bytes);
        let (gpu_rgb, gw, gh) = decode_jpeg_rgb(&gpu_out.bytes);
        if (cw, ch) != (gw, gh) {
            panic!("range mask decoded dim mismatch {cw}x{ch} vs {gw}x{gh}");
        }
        let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
        eprintln!("range masks: mean abs delta = {delta:.3}");
        if delta > 2.5 {
            panic!("CPU vs GPU range mask drift: {delta:.3} > 2.5");
        }
    }
}

fn warn_pixels(rgb: &[u8], color: [u8; 3]) -> Vec<usize> {
    rgb.chunks_exact(3)
        .enumerate()
        .filter(|(_, p)| p[0] == color[0] && p[1] == color[1] && p[2] == color[2])
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn gpu_matches_cpu_for_clipping_warnings() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    for (exposure, saturation, color) in [
        (3.0, 0.0, raw_pipeline::warn::HIGHLIGHT_WARN_RGB),
        (-14.0, 0.0, raw_pipeline::warn::SHADOW_WARN_RGB),
        (0.0, 100.0, raw_pipeline::warn::GAMUT_WARN_RGB),
    ] {
        let edits = Edits {
            basic: BasicEdits {
                exposure_ev: exposure,
                saturation,
                ..Default::default()
            },
            ..Default::default()
        };
        let opts = RenderOptions {
            max_edge: 96,
            output: OutputFormat::Rgb8,
            clip_warn: true,
            gamut_warn: true,
            ..Default::default()
        };
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let c = warn_pixels(&cpu.bytes, color);
        let g = warn_pixels(&gpu.bytes, color);
        if c.is_empty() {
            panic!("cpu produced no {color:?} warning pixels; test is vacuous");
        }
        let only_cpu = c.iter().filter(|i| !g.contains(i)).count();
        let only_gpu = g.iter().filter(|i| !c.contains(i)).count();
        let drift = (only_cpu + only_gpu) as f64 / c.len() as f64;
        eprintln!(
            "{color:?}: cpu={} gpu={} disagree={} ({drift:.4})",
            c.len(),
            g.len(),
            only_cpu + only_gpu
        );
        if drift > 0.01 {
            panic!("warning drift for {color:?}: {drift:.4} > 0.01");
        }
    }
}
