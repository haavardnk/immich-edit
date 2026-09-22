use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RenderOptions, RenderedImage};
use raw_pipeline::timing::{DEMOSAIC, DISPLAY, ENCODE, FINISH, READBACK};
use raw_pipeline::{GpuRenderer, GpuRendererOptions, cpu};

mod common;

use common::synthetic_bayer_frame;

fn opts() -> RenderOptions {
    RenderOptions {
        max_edge: 64,
        ..Default::default()
    }
}

fn require_stages(label: &str, image: &RenderedImage, expected: &[&str]) {
    let got: Vec<&str> = image.timings.iter().map(|t| t.stage).collect();
    let missing: Vec<&&str> = expected.iter().filter(|s| !got.contains(s)).collect();
    if !missing.is_empty() {
        panic!("{label} render is missing stages {missing:?}, got {got:?}");
    }
    if got.iter().enumerate().any(|(i, s)| got[..i].contains(s)) {
        panic!("{label} render reported a stage twice: {got:?}");
    }
}

#[test]
fn cpu_render_reports_each_stage() {
    let frame = synthetic_bayer_frame(64, 48, "RGGB");
    let image = cpu::render(&frame, &Edits::default(), &opts()).unwrap();
    require_stages("cpu", &image, &[DEMOSAIC, FINISH, ENCODE]);
    if image.timings.iter().any(|t| t.gpu.is_some()) {
        panic!("cpu stages carry device time: {:?}", image.timings);
    }
}

#[test]
fn gpu_render_reports_device_time_only_with_timestamps() {
    let renderer = match GpuRenderer::with_options(GpuRendererOptions {
        timestamps: true,
        ..Default::default()
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            return;
        }
    };
    let frame = synthetic_bayer_frame(64, 48, "RGGB");
    let image = renderer.render(&frame, &Edits::default(), &opts()).unwrap();
    require_stages("gpu", &image, &[DEMOSAIC, DISPLAY, READBACK, ENCODE]);

    let timed: Vec<&str> = image
        .timings
        .iter()
        .filter(|t| t.gpu.is_some())
        .map(|t| t.stage)
        .collect();
    if !renderer.gpu_timestamps() {
        eprintln!("adapter lacks TIMESTAMP_QUERY; checking wall time only");
        if !timed.is_empty() {
            panic!("stages {timed:?} carry device time without timestamp queries");
        }
        return;
    }
    if !timed.contains(&DEMOSAIC) || !timed.contains(&DISPLAY) {
        panic!("device time missing on demosaic or display, timed {timed:?}");
    }
    let display = image
        .timings
        .iter()
        .find(|t| t.stage == DISPLAY)
        .and_then(|t| t.gpu)
        .unwrap_or_default();
    if display.is_zero() {
        panic!(
            "display pass reported zero device time: {:?}",
            image.timings
        );
    }
    if timed.contains(&ENCODE) {
        panic!("encode runs on the cpu but carries device time");
    }
}
