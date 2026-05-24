use raw_pipeline::edits::BasicEdits;
use raw_pipeline::edits::{CropRect, GeometryEdits};
use raw_pipeline::frame::RawFrame;
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
            1.0,
            Edits {
                basic: BasicEdits {
                    contrast: 30.0,
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
        exif: None,
    };
    let mut edits = Edits::default();
    edits.basic.texture = 30.0;
    edits.basic.clarity = 20.0;
    edits.basic.dehaze = 15.0;

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
