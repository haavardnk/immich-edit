use raw_pipeline::{GpuRenderer, decode, edits::Edits, frame::RenderOptions};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    if path.exists() { Some(path) } else { None }
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
    let Some(path) = fixture("sample.arw").or_else(|| fixture("sample.dng")) else {
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
    if out.jpeg.len() < 1000 {
        panic!("jpeg too small ({} bytes)", out.jpeg.len());
    }
    if &out.jpeg[..2] != b"\xff\xd8" {
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
    let Some(path) = fixture("sample.arw").or_else(|| fixture("sample.dng")) else {
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
    let Some(path) = fixture("sample.arw").or_else(|| fixture("sample.dng")) else {
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

#[test]
fn gpu_exif_orientation_matches_cpu() {
    use raw_pipeline::frame::RawFrame;

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
            cam_to_xyz: [[0.0; 4]; 3],
            black_levels: [0.0; 4],
            white_levels: [1.0; 4],
            data: data.clone(),
            cpp: 3,
            orientation: orient,
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
