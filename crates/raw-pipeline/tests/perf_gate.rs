use raw_pipeline::cpu::run_pipeline_ops;
use raw_pipeline::edits::{
    BasicEdits, CurvePoint, CurvePoints, CurvesEdits, DetailEdits, Edits, ToneEdits,
};
use raw_pipeline::ops::{LinearImage, OpContext, OpScratch, RenderContext};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

const W: usize = 512;
const H: usize = 512;
const ITERS: usize = 7;
const REGRESSION_FACTOR: f64 = 2.0;

#[derive(Serialize, Deserialize, PartialEq)]
struct PerfBaseline {
    median_ns: BTreeMap<String, u64>,
}

fn baseline_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/baselines/perf_gate.json")
}

fn make_image(w: usize, h: usize) -> LinearImage {
    let mut rgb = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / w as f32;
            let v = y as f32 / h as f32;
            rgb.push(0.2 + 0.6 * u);
            rgb.push(0.15 + 0.7 * v);
            rgb.push(0.25 + 0.5 * (u * v));
        }
    }
    LinearImage::new(rgb, w, h)
}

fn ctx() -> OpContext {
    OpContext {
        render: RenderContext {
            wb_coeffs: [2.1, 1.0, 1.45, 1.0],
            cam_to_srgb: [
                [1.85, -0.65, -0.20],
                [-0.20, 1.45, -0.25],
                [0.02, -0.45, 1.43],
            ],
            is_raw: true,
            preview_mode: raw_pipeline::frame::PreviewMode::None,
        },
        scratch: OpScratch { shadows_blur: None },
    }
}

fn edits_identity() -> Edits {
    Edits::default()
}

fn edits_typical() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.4,
            contrast: 25.0,
            saturation: 10.0,
            curves: CurvesEdits {
                composite: CurvePoints {
                    points: vec![
                        CurvePoint { x: 0.0, y: 0.05 },
                        CurvePoint { x: 0.5, y: 0.55 },
                        CurvePoint { x: 1.0, y: 0.98 },
                    ],
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

fn edits_full() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.6,
            brightness: 20.0,
            contrast: 30.0,
            saturation: 15.0,
            vibrance: 25.0,
            wb_temp: 12.0,
            wb_tint: -5.0,
            texture: 20.0,
            clarity: 15.0,
            dehaze: 10.0,
            curves: CurvesEdits {
                composite: CurvePoints {
                    points: vec![
                        CurvePoint { x: 0.0, y: 0.02 },
                        CurvePoint { x: 0.3, y: 0.28 },
                        CurvePoint { x: 0.7, y: 0.75 },
                        CurvePoint { x: 1.0, y: 0.96 },
                    ],
                },
                ..Default::default()
            },
        },
        tone: ToneEdits {
            highlights: -30.0,
            shadows: 25.0,
            blacks: 10.0,
            whites: -15.0,
        },
        ..Default::default()
    }
}

fn edits_detail_heavy() -> Edits {
    Edits {
        detail: DetailEdits {
            sharpen_amount: 80.0,
            sharpen_radius: 1.2,
            sharpen_detail: 40.0,
            sharpen_masking: 30.0,
            luma_nr_amount: 60.0,
            luma_nr_detail: 50.0,
            luma_nr_contrast: 20.0,
            color_nr_amount: 50.0,
            color_nr_detail: 50.0,
            color_nr_smoothness: 25.0,
        },
        ..Default::default()
    }
}

fn measure(label: &str, edits: &Edits, base: &LinearImage) -> u64 {
    let ctx = ctx();
    let rasters = raw_pipeline::empty_rasters();
    for _ in 0..2 {
        let mut img = LinearImage::new(base.rgb.clone(), base.width, base.height);
        run_pipeline_ops(&mut img, &ctx, edits, &rasters, None).unwrap();
    }
    let mut samples: Vec<u64> = Vec::with_capacity(ITERS);
    for _ in 0..ITERS {
        let mut img = LinearImage::new(base.rgb.clone(), base.width, base.height);
        let t = Instant::now();
        run_pipeline_ops(&mut img, &ctx, edits, &rasters, None).unwrap();
        samples.push(t.elapsed().as_nanos() as u64);
    }
    samples.sort();
    let median = samples[samples.len() / 2];
    eprintln!(
        "perf {label}: median {median} ns ({} samples)",
        samples.len()
    );
    median
}

#[test]
fn perf_gate_pipeline_profiles() {
    if std::env::var_os("PERF_GATE").is_none() && std::env::var_os("BAKE_PERF_GATE").is_none() {
        eprintln!("skip: set PERF_GATE=1 to enforce, BAKE_PERF_GATE=1 to regenerate baseline");
        return;
    }
    let base = make_image(W, H);
    let profiles: [(&str, Edits); 4] = [
        ("identity", edits_identity()),
        ("typical", edits_typical()),
        ("full", edits_full()),
        ("detail_heavy", edits_detail_heavy()),
    ];
    let measured: BTreeMap<String, u64> = profiles
        .iter()
        .map(|(name, e)| ((*name).to_string(), measure(name, e, &base)))
        .collect();

    if std::env::var_os("BAKE_PERF_GATE").is_some() {
        let bl = PerfBaseline {
            median_ns: measured,
        };
        let json = serde_json::to_string_pretty(&bl).unwrap();
        std::fs::write(baseline_path(), json).unwrap();
        eprintln!("baked perf baseline to {}", baseline_path().display());
        return;
    }

    let raw = std::fs::read_to_string(baseline_path())
        .expect("perf_gate.json missing — run BAKE_PERF_GATE=1");
    let baseline: PerfBaseline = serde_json::from_str(&raw).unwrap();
    let mut failures: Vec<String> = Vec::new();
    for (name, observed) in &measured {
        let baseline_ns = baseline
            .median_ns
            .get(name)
            .copied()
            .unwrap_or_else(|| panic!("baseline missing entry for {name}"));
        let ratio = *observed as f64 / baseline_ns as f64;
        eprintln!(
            "perf {name}: ratio {:.2}x (baseline {} ns)",
            ratio, baseline_ns
        );
        if ratio > REGRESSION_FACTOR {
            failures.push(format!(
                "{name}: {observed} ns > {:.1}x baseline {baseline_ns} ns",
                REGRESSION_FACTOR
            ));
        }
    }
    if !failures.is_empty() {
        panic!("perf regressions:\n  {}", failures.join("\n  "));
    }
}
