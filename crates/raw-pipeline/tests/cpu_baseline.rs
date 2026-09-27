use raw_pipeline::{cpu, decode, edits::Edits, frame::RenderOptions};
use raw_pipeline_testkit::baseline::{RenderStats, Tolerance};
use raw_pipeline_testkit::fixtures::{baseline_path, fixtures};
use raw_pipeline_testkit::render::decode_jpeg_rgb;
use std::collections::BTreeMap;
use std::path::Path;

const MAX_EDGE: u32 = 512;
const TOLERANCE: Tolerance = Tolerance {
    mean_rgb: 1.0,
    grid_cell: 3.0,
    grid_mean: 1.0,
};

type BaselineMap = BTreeMap<String, RenderStats>;

fn render_metrics(path: &Path) -> Option<RenderStats> {
    let bytes = std::fs::read(path).ok()?;
    let frame = decode::decode(&bytes).ok()?;
    let opts = RenderOptions {
        max_edge: MAX_EDGE,
        ..Default::default()
    };
    let out = cpu::render(&frame, &Edits::default(), &opts).ok()?;
    let (rgb, w, h) = decode_jpeg_rgb(&out.bytes);
    Some(RenderStats::measure(&rgb, w, h))
}

#[test]
fn cpu_baseline_per_fixture() {
    let paths = fixtures();
    if paths.is_empty() {
        eprintln!("no fixtures; skipping");
        return;
    }
    let bake = std::env::var("BAKE_BASELINE").ok().as_deref() == Some("1");
    let baseline_file = baseline_path("cpu_baseline.json");
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
            let errs = m.drift(base, &TOLERANCE);
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
