use raw_pipeline::{decode, edits::Edits, frame::RenderOptions, CpuRenderer, Renderer};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    if path.exists() { Some(path) } else { None }
}

fn each_fixture(test: impl Fn(&str, &PathBuf)) {
    let names = ["sample.dng", "sample.arw"];
    let mut ran = 0;
    for name in names {
        if let Some(p) = fixture(name) {
            test(name, &p);
            ran += 1;
        }
    }
    if ran == 0 {
        eprintln!("no fixtures found; skipping");
    }
}

#[test]
fn decode_metadata() {
    each_fixture(|name, path| {
        let bytes = std::fs::read(path).unwrap();
        let frame = decode::decode(&bytes).unwrap_or_else(|e| panic!("{name}: decode failed: {e}"));
        if frame.width == 0 || frame.height == 0 {
            panic!("{name}: zero dim");
        }
        if frame.cfa_pattern.is_empty() && frame.cpp == 1 {
            panic!("{name}: bayer without cfa pattern");
        }
        if frame.bps == 0 || frame.bps > 16 {
            panic!("{name}: bad bps {}", frame.bps);
        }
        if frame.data.is_empty() {
            panic!("{name}: no pixel data");
        }
    });
}

#[test]
fn identity_render_jpeg() {
    each_fixture(|name, path| {
        let bytes = std::fs::read(path).unwrap();
        let frame = decode::decode(&bytes).unwrap();
        let renderer = CpuRenderer;
        let opts = RenderOptions { max_edge: 512 };
        let out = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        if out.jpeg.len() < 1000 {
            panic!("{name}: jpeg too small ({} bytes)", out.jpeg.len());
        }
        if &out.jpeg[..2] != b"\xff\xd8" {
            panic!("{name}: not jpeg SOI marker");
        }
        if out.width.max(out.height) > 512 {
            panic!("{name}: max edge exceeded {}x{}", out.width, out.height);
        }
        if out.histogram.pixel_count() != (out.width as u64) * (out.height as u64) {
            panic!("{name}: histogram pixel count mismatch");
        }
    });
}

#[test]
fn rotate_swaps_dims() {
    each_fixture(|name, path| {
        let bytes = std::fs::read(path).unwrap();
        let frame = decode::decode(&bytes).unwrap();
        let renderer = CpuRenderer;
        let opts = RenderOptions { max_edge: 256 };
        let base = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let rotated_edits = Edits { rotate: 90, ..Default::default() };
        let rotated = renderer.render(&frame, &rotated_edits, &opts).unwrap();
        if base.width == base.height {
            return;
        }
        if rotated.width != base.height || rotated.height != base.width {
            panic!("{name}: rotate90 dims {} {} -> {} {}", base.width, base.height, rotated.width, rotated.height);
        }
    });
}

#[test]
fn exposure_raises_mean() {
    each_fixture(|name, path| {
        let bytes = std::fs::read(path).unwrap();
        let frame = decode::decode(&bytes).unwrap();
        let renderer = CpuRenderer;
        let opts = RenderOptions { max_edge: 256 };
        let base = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let bright_edits = Edits { exposure_ev: 2.0, ..Default::default() };
        let bright = renderer.render(&frame, &bright_edits, &opts).unwrap();
        let base_mean = histogram_mean(&base.histogram.l);
        let bright_mean = histogram_mean(&bright.histogram.l);
        if bright_mean <= base_mean {
            panic!("{name}: exposure +2 mean {} <= base {}", bright_mean, base_mean);
        }
    });
}

#[test]
fn crop_reduces_dims() {
    each_fixture(|name, path| {
        let bytes = std::fs::read(path).unwrap();
        let frame = decode::decode(&bytes).unwrap();
        let renderer = CpuRenderer;
        let opts = RenderOptions { max_edge: 4096 };
        let base = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let crop_edits = Edits {
            crop: Some(raw_pipeline::edits::CropRect {
                x: 0.25,
                y: 0.25,
                width: 0.5,
                height: 0.5,
            }),
            ..Default::default()
        };
        let cropped = renderer.render(&frame, &crop_edits, &opts).unwrap();
        if cropped.width >= base.width || cropped.height >= base.height {
            panic!("{name}: crop did not reduce dims");
        }
    });
}

#[test]
fn orientation_swaps_display_dims_when_transposed() {
    each_fixture(|name, path| {
        let bytes = std::fs::read(path).unwrap();
        let frame = decode::decode(&bytes).unwrap();
        let renderer = CpuRenderer;
        let opts = RenderOptions { max_edge: 256 };
        let out = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let (transpose, _, _) = frame.orientation;
        let (expected_w, expected_h) = if transpose {
            (frame.height, frame.width)
        } else {
            (frame.width, frame.height)
        };
        let landscape_sensor = expected_w > expected_h;
        let landscape_out = out.width > out.height;
        if landscape_sensor != landscape_out && out.width != out.height {
            panic!(
                "{name}: oriented landscape={landscape_sensor} but out landscape={landscape_out} ({}x{})",
                out.width, out.height
            );
        }
    });
}

fn histogram_mean(bins: &[u32]) -> f64 {
    let total: u64 = bins.iter().map(|&v| v as u64).sum();
    if total == 0 {
        return 0.0;
    }
    let weighted: u64 = bins.iter().enumerate().map(|(i, &v)| i as u64 * v as u64).sum();
    weighted as f64 / total as f64
}
