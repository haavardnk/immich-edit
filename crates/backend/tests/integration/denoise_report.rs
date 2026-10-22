use std::path::{Path, PathBuf};
use std::time::Instant;

use ml::runtime::SessionConfig;
use ml::{Denoiser, RuntimeMode};
use raw_pipeline::PipelineError;
use raw_pipeline::ai_denoise::{DenoisePlane, ModelSpace};
use raw_pipeline::edits::{CropRect, Edits};
use raw_pipeline::frame::{OutputFormat, RawFrame, RenderOptions};

fn model_path() -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ml/tests/fixtures/nind_denoise_utnet_684.onnx");
    path.exists().then_some(path)
}

fn raws() -> Vec<PathBuf> {
    let dir = std::env::var("DENOISE_RAWS").map_or_else(
        |_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../raw-pipeline/tests/fixtures"),
        PathBuf::from,
    );
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.extension().and_then(|e| e.to_str()).is_some_and(|e| {
                [
                    "arw", "cr2", "cr3", "nef", "raf", "orf", "rw2", "dng", "pef",
                ]
                .contains(&e.to_ascii_lowercase().as_str())
            })
        })
        .collect();
    paths.sort();
    paths
}

fn mean_linear_luma(
    frame: &RawFrame,
    space: &ModelSpace,
    rgb: impl Iterator<Item = [f32; 3]>,
) -> f64 {
    let total: f64 = rgb
        .map(|px| {
            let lin = space.linear_input(px);
            f64::from(0.2126 * lin[0] + 0.7152 * lin[1] + 0.0722 * lin[2])
        })
        .sum();
    total / (frame.meta.width * frame.meta.height) as f64
}

fn center_crop(frame: &RawFrame) -> CropRect {
    let w = (768.0 / frame.meta.width as f32).min(1.0);
    let h = (512.0 / frame.meta.height as f32).min(1.0);
    CropRect {
        x: (1.0 - w) / 2.0,
        y: (1.0 - h) / 2.0,
        w,
        h,
    }
}

#[test]
fn denoise_report() {
    if std::env::var("DENOISE_REPORT").is_err() {
        return;
    }
    let Some(model) = model_path() else {
        eprintln!("skipping denoise report: model fixture missing");
        return;
    };
    let mode = match std::env::var("DENOISE_RUNTIME").as_deref() {
        Ok("cpu") => RuntimeMode::Cpu,
        _ => RuntimeMode::Auto,
    };
    let out = std::env::var("DENOISE_OUT").ok().map(PathBuf::from);
    let mut denoiser = Denoiser::open(&model, mode, &SessionConfig::default()).unwrap();
    println!("| raw | MP | tiles | bake s | backend | luma before | luma after | drift % |");
    println!("|---|---|---|---|---|---|---|---|");
    for path in raws() {
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        let bytes = std::fs::read(&path).unwrap();
        let Ok(frame) = raw_pipeline::decode::decode_quality(&bytes) else {
            continue;
        };
        if !frame.meta.is_raw {
            continue;
        }
        let edge = denoiser.tile_edge();
        let mut tiles = 0;
        let started = Instant::now();
        let plane = raw_pipeline::ai_denoise::bake(
            &frame,
            1,
            edge,
            |tile| {
                denoiser
                    .run(tile)
                    .map_err(|e| PipelineError::Render(e.to_string()))
            },
            |_, total| tiles = total,
        )
        .unwrap();
        let elapsed = started.elapsed().as_secs_f64();
        let space = ModelSpace::for_frame(&frame.meta).unwrap();
        let original =
            raw_pipeline::ai_denoise::bake(&frame, 0, edge, |tile| Ok(tile.to_vec()), |_, _| {})
                .unwrap();
        let before = mean_linear_luma(&frame, &space, pixels(&original));
        let after = mean_linear_luma(&frame, &space, pixels(&plane));
        println!(
            "| {name} | {:.1} | {tiles} | {elapsed:.1} | {} | {before:.4} | {after:.4} | {:+.2} |",
            (frame.meta.width * frame.meta.height) as f64 / 1e6,
            denoiser.backend().as_str(),
            (after / before - 1.0) * 100.0
        );
        if let Some(dir) = &out {
            write_crops(dir, &name, &frame, plane);
        }
    }
}

fn pixels(plane: &DenoisePlane) -> impl Iterator<Item = [f32; 3]> + '_ {
    (0..plane.height).flat_map(move |y| (0..plane.width).map(move |x| plane.binned_pixel(x, y, 1)))
}

fn write_crops(dir: &Path, name: &str, frame: &RawFrame, plane: DenoisePlane) {
    std::fs::create_dir_all(dir).unwrap();
    let plane = std::sync::Arc::new(plane);
    for strength in [0.0f64, 50.0, 75.0, 100.0] {
        let mut edits = Edits::default();
        edits.detail.ai_denoise = strength > 0.0;
        edits.detail.ai_denoise_strength = strength.max(1.0);
        edits.geometry.crop = Some(center_crop(frame));
        let opts = RenderOptions {
            max_edge: 65535,
            quality: true,
            output: OutputFormat::Jpeg {
                quality: 95,
                subsampling: raw_pipeline::frame::JpegSubsampling::Chroma444,
            },
            denoise: Some(plane.clone()),
            ..Default::default()
        };
        let rendered = raw_pipeline::cpu::render(frame, &edits, &opts).unwrap();
        std::fs::write(
            dir.join(format!("{name}-{strength:03}.jpg")),
            rendered.bytes,
        )
        .unwrap();
    }
}
