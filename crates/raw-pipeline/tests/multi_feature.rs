use raw_pipeline::{
    cpu, decode,
    edits::{BasicEdits, DetailEdits, Edits, ToneEdits},
    frame::RenderOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const FIXTURE: &str = "Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw";
const MAX_EDGE: u32 = 512;
const GRID: usize = 8;
const MEAN_RGB_TOL: f64 = 1.5;
const GRID_CELL_TOL: f64 = 4.0;
const GRID_MEAN_TOL: f64 = 1.5;

#[derive(Serialize, Deserialize, PartialEq)]
struct StackBaseline {
    width: u32,
    height: u32,
    mean_rgb: [f64; 3],
    luma_grid: Vec<f64>,
}

type BaselineMap = BTreeMap<String, StackBaseline>;

fn baseline_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/baselines/multi_feature.json")
}

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(FIXTURE)
}

fn stack_tone_lift() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.25,
            contrast: 12.0,
            saturation: 4.0,
            vibrance: 18.0,
            wb_temp: 8.0,
            wb_tint: 3.0,
            clarity: -8.0,
            texture: -5.0,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -25.0,
            shadows: 30.0,
            blacks: 8.0,
            whites: -10.0,
        },
        detail: DetailEdits {
            sharpen_amount: 35.0,
            sharpen_masking: 45.0,
            color_nr_amount: 25.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn stack_clarity_dehaze() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: -0.1,
            contrast: 22.0,
            saturation: 8.0,
            vibrance: 30.0,
            wb_temp: -6.0,
            clarity: 25.0,
            texture: 15.0,
            dehaze: 12.0,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -35.0,
            shadows: 18.0,
            blacks: -5.0,
            whites: 8.0,
        },
        detail: DetailEdits {
            sharpen_amount: 60.0,
            sharpen_radius: 1.0,
            sharpen_detail: 35.0,
            sharpen_masking: 20.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn stack_noise_reduction() -> Edits {
    Edits {
        basic: BasicEdits {
            exposure_ev: 0.6,
            contrast: 18.0,
            saturation: -10.0,
            vibrance: 15.0,
            wb_temp: 4.0,
            clarity: 10.0,
            dehaze: 8.0,
            ..Default::default()
        },
        tone: ToneEdits {
            highlights: -15.0,
            shadows: 40.0,
            blacks: 12.0,
            whites: -5.0,
        },
        detail: DetailEdits {
            sharpen_amount: 25.0,
            luma_nr_amount: 55.0,
            luma_nr_detail: 40.0,
            color_nr_amount: 60.0,
            color_nr_smoothness: 60.0,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn decode_jpeg_rgb(jpeg: &[u8]) -> (Vec<u8>, usize, usize) {
    let img: turbojpeg::Image<Vec<u8>> =
        turbojpeg::decompress(jpeg, turbojpeg::PixelFormat::RGB).unwrap();
    (img.pixels, img.width, img.height)
}

fn compute_metrics(rgb: &[u8], width: usize, height: usize) -> StackBaseline {
    let n = (width * height) as f64;
    let mut sum: [f64; 3] = [0.0, 0.0, 0.0];
    for px in rgb.chunks_exact(3) {
        sum[0] += px[0] as f64;
        sum[1] += px[1] as f64;
        sum[2] += px[2] as f64;
    }
    let mean_rgb = [sum[0] / n, sum[1] / n, sum[2] / n];
    let mut grid = vec![0.0_f64; GRID * GRID];
    let mut counts = vec![0u32; GRID * GRID];
    for y in 0..height {
        for x in 0..width {
            let gx = (x * GRID / width).min(GRID - 1);
            let gy = (y * GRID / height).min(GRID - 1);
            let i = (y * width + x) * 3;
            let l =
                0.2126 * rgb[i] as f64 + 0.7152 * rgb[i + 1] as f64 + 0.0722 * rgb[i + 2] as f64;
            grid[gy * GRID + gx] += l;
            counts[gy * GRID + gx] += 1;
        }
    }
    for (g, c) in grid.iter_mut().zip(counts.iter()) {
        if *c > 0 {
            *g /= *c as f64;
        }
    }
    StackBaseline {
        width: width as u32,
        height: height as u32,
        mean_rgb,
        luma_grid: grid,
    }
}

fn render_stack(edits: &Edits) -> StackBaseline {
    let bytes = std::fs::read(fixture_path()).expect("fixture missing");
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: MAX_EDGE,
        ..Default::default()
    };
    let out = cpu::render(&frame, edits, &opts).unwrap();
    let (rgb, w, h) = decode_jpeg_rgb(&out.bytes);
    compute_metrics(&rgb, w, h)
}

#[test]
fn multi_feature_stacks() {
    if !fixture_path().exists() {
        eprintln!("skip: {FIXTURE} missing");
        return;
    }
    let stacks: [(&str, Edits); 3] = [
        ("tone_lift", stack_tone_lift()),
        ("clarity_dehaze", stack_clarity_dehaze()),
        ("noise_reduction", stack_noise_reduction()),
    ];
    let measured: BaselineMap = stacks
        .iter()
        .map(|(name, e)| ((*name).to_string(), render_stack(e)))
        .collect();

    if std::env::var_os("BAKE_MULTI_FEATURE").is_some() {
        let json = serde_json::to_string_pretty(&measured).unwrap();
        std::fs::write(baseline_path(), json).unwrap();
        eprintln!("baked multi-feature baseline ({} stacks)", measured.len());
        return;
    }

    let raw = std::fs::read_to_string(baseline_path())
        .expect("multi_feature.json missing — run BAKE_MULTI_FEATURE=1");
    let baseline: BaselineMap = serde_json::from_str(&raw).unwrap();
    let mut failures: Vec<String> = Vec::new();
    for (name, got) in &measured {
        let want = baseline
            .get(name)
            .unwrap_or_else(|| panic!("baseline missing stack {name}"));
        if (got.width, got.height) != (want.width, want.height) {
            failures.push(format!(
                "{name}: dims ({}x{}) vs baseline ({}x{})",
                got.width, got.height, want.width, want.height
            ));
            continue;
        }
        let drgb: [f64; 3] = [
            (got.mean_rgb[0] - want.mean_rgb[0]).abs(),
            (got.mean_rgb[1] - want.mean_rgb[1]).abs(),
            (got.mean_rgb[2] - want.mean_rgb[2]).abs(),
        ];
        if drgb.iter().any(|d| *d > MEAN_RGB_TOL) {
            failures.push(format!(
                "{name}: mean_rgb drift {:?} > tol {MEAN_RGB_TOL}",
                drgb
            ));
        }
        let cell_diffs: Vec<f64> = got
            .luma_grid
            .iter()
            .zip(want.luma_grid.iter())
            .map(|(a, b)| (a - b).abs())
            .collect();
        let max_cell = cell_diffs.iter().cloned().fold(0.0_f64, f64::max);
        let mean_cell = cell_diffs.iter().sum::<f64>() / cell_diffs.len() as f64;
        if max_cell > GRID_CELL_TOL {
            failures.push(format!(
                "{name}: grid cell drift {:.2} > tol {GRID_CELL_TOL}",
                max_cell
            ));
        }
        if mean_cell > GRID_MEAN_TOL {
            failures.push(format!(
                "{name}: grid mean drift {:.2} > tol {GRID_MEAN_TOL}",
                mean_cell
            ));
        }
    }
    if !failures.is_empty() {
        panic!("multi-feature regressions:\n  {}", failures.join("\n  "));
    }
}
