use raw_pipeline::{cpu, decode, edits::Edits, frame::RenderOptions};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const RAW_EXTS: &[&str] = &[
    "arw", "cr2", "cr3", "crw", "dng", "erf", "gpr", "iiq", "mrw", "nef", "nrw", "orf", "pef",
    "raf", "raw", "rw2", "rwl", "sr2", "srw", "x3f",
];

const MAX_EDGE: u32 = 512;
const GRID: usize = 8;

const MEAN_RGB_TOL: f64 = 1.0;
const GRID_CELL_TOL: f64 = 3.0;
const GRID_MEAN_TOL: f64 = 1.0;

#[derive(Serialize, Deserialize, PartialEq)]
struct Baseline {
    width: u32,
    height: u32,
    mean_rgb: [f64; 3],
    luma_grid: Vec<f64>,
}

type BaselineMap = BTreeMap<String, Baseline>;

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn baseline_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/baselines/cpu_baseline.json")
}

fn fixtures() -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(fixtures_dir()) else {
        return Vec::new();
    };
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
    paths
}

fn decode_jpeg_rgb(jpeg: &[u8]) -> (Vec<u8>, usize, usize) {
    let img: turbojpeg::Image<Vec<u8>> =
        turbojpeg::decompress(jpeg, turbojpeg::PixelFormat::RGB).unwrap();
    (img.pixels, img.width, img.height)
}

fn compute_metrics(rgb: &[u8], width: usize, height: usize) -> Baseline {
    let n = (width * height) as f64;
    let mut sum = [0.0f64; 3];
    for px in rgb.chunks_exact(3) {
        sum[0] += px[0] as f64;
        sum[1] += px[1] as f64;
        sum[2] += px[2] as f64;
    }
    let mean_rgb: [f64; 3] = [sum[0] / n, sum[1] / n, sum[2] / n];

    let mut grid_sum = vec![0.0f64; GRID * GRID];
    let mut grid_count = vec![0u32; GRID * GRID];
    for y in 0..height {
        let gy = (y * GRID / height).min(GRID - 1);
        for x in 0..width {
            let gx = (x * GRID / width).min(GRID - 1);
            let i = (y * width + x) * 3;
            let l =
                0.2126 * rgb[i] as f64 + 0.7152 * rgb[i + 1] as f64 + 0.0722 * rgb[i + 2] as f64;
            grid_sum[gy * GRID + gx] += l;
            grid_count[gy * GRID + gx] += 1;
        }
    }
    let luma_grid: Vec<f64> = grid_sum
        .iter()
        .zip(grid_count.iter())
        .map(|(s, c)| if *c == 0 { 0.0 } else { s / *c as f64 })
        .collect();

    Baseline {
        width: width as u32,
        height: height as u32,
        mean_rgb,
        luma_grid,
    }
}

fn render_metrics(path: &Path) -> Option<Baseline> {
    let bytes = std::fs::read(path).ok()?;
    let frame = decode::decode(&bytes).ok()?;
    let opts = RenderOptions {
        max_edge: MAX_EDGE,
        ..Default::default()
    };
    let out = cpu::render(&frame, &Edits::default(), &opts).ok()?;
    let (rgb, w, h) = decode_jpeg_rgb(&out.bytes);
    Some(compute_metrics(&rgb, w, h))
}

fn diff(current: &Baseline, base: &Baseline) -> Vec<String> {
    let mut errs: Vec<String> = Vec::new();
    if current.width != base.width || current.height != base.height {
        errs.push(format!(
            "dims {}x{} != baseline {}x{}",
            current.width, current.height, base.width, base.height
        ));
        return errs;
    }
    for c in 0..3 {
        let d = (current.mean_rgb[c] - base.mean_rgb[c]).abs();
        if d > MEAN_RGB_TOL {
            errs.push(format!("mean ch{c} diff {d:.3} > {MEAN_RGB_TOL}"));
        }
    }
    if current.luma_grid.len() != base.luma_grid.len() {
        errs.push("grid length mismatch".into());
        return errs;
    }
    let mut max_cell = 0.0f64;
    let mut sum_cell = 0.0f64;
    for (a, b) in current.luma_grid.iter().zip(base.luma_grid.iter()) {
        let d = (a - b).abs();
        if d > max_cell {
            max_cell = d;
        }
        sum_cell += d;
    }
    let mean_cell = sum_cell / current.luma_grid.len() as f64;
    if max_cell > GRID_CELL_TOL {
        errs.push(format!(
            "luma grid max cell diff {max_cell:.3} > {GRID_CELL_TOL}"
        ));
    }
    if mean_cell > GRID_MEAN_TOL {
        errs.push(format!(
            "luma grid mean diff {mean_cell:.3} > {GRID_MEAN_TOL}"
        ));
    }
    errs
}

#[test]
fn cpu_baseline_per_fixture() {
    let paths = fixtures();
    if paths.is_empty() {
        eprintln!("no fixtures; skipping");
        return;
    }
    let bake = std::env::var("BAKE_BASELINE").ok().as_deref() == Some("1");
    let baseline_file = baseline_path();
    let existing: BaselineMap = if baseline_file.exists() {
        let bytes = std::fs::read(&baseline_file).expect("read baseline");
        serde_json::from_slice(&bytes).expect("parse baseline")
    } else {
        BaselineMap::new()
    };

    let mut current: BaselineMap = BaselineMap::new();
    let mut failed: Vec<String> = Vec::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        let Some(m) = render_metrics(p) else {
            eprintln!("skip {name}: decode or render failed");
            continue;
        };
        if !bake && let Some(base) = existing.get(&name) {
            let errs = diff(&m, base);
            if !errs.is_empty() {
                failed.push(format!("{name}: {}", errs.join("; ")));
            }
        }
        current.insert(name, m);
    }

    if bake {
        if let Some(dir) = baseline_file.parent() {
            std::fs::create_dir_all(dir).expect("create baseline dir");
        }
        let json = serde_json::to_string_pretty(&current).expect("serialize baseline");
        std::fs::write(&baseline_file, json).expect("write baseline");
        eprintln!("baked baseline with {} fixtures", current.len());
        return;
    }

    let missing: Vec<&String> = current
        .keys()
        .filter(|k| !existing.contains_key(*k))
        .collect();
    if !missing.is_empty() {
        panic!(
            "no baseline for: {}. Run with BAKE_BASELINE=1 to generate.",
            missing
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if !failed.is_empty() {
        panic!("cpu baseline regressions:\n  {}", failed.join("\n  "));
    }
}
