use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::CpuRenderer;
use crate::PipelineError;
use crate::admission::{HostMemory, Retained};
use crate::cancel::CancelTracker;
use crate::cpu::estimate_render;
use crate::edits::Edits;
use crate::frame::{FrameId, FrameMeta, OutputFormat, RawFrame, RenderOptions};

fn mosaic(width: usize, height: usize) -> RawFrame {
    let data = (0..width * height)
        .map(|i| {
            if (i % width / 4 + i / width / 4).is_multiple_of(2) {
                0.2
            } else {
                0.8
            }
        })
        .collect();
    RawFrame {
        meta: FrameMeta {
            width,
            height,
            wb_coeffs: [2.0, 1.0, 1.5, 1.0],
            xyz_to_cam: [[0.0; 3]; 4],
            color_matrices: Vec::new(),
            orientation: (false, false, false),
            is_raw: true,
            capture_sigma: None,
            model: String::new(),
        },
        cfa_pattern: "RGGB".into(),
        bps: 16,
        data,
        cpp: 1,
        exif: None,
        embedded_lens: Default::default(),
        id: FrameId::fresh(),
    }
}

fn preview() -> RenderOptions {
    RenderOptions {
        max_edge: 600,
        output: OutputFormat::Rgb8,
        ..RenderOptions::default()
    }
}

fn budgeted(bytes: u64) -> CpuRenderer {
    CpuRenderer::with_host(HostMemory::new(Some(bytes)))
}

#[test]
fn a_render_over_the_memory_budget_is_refused_and_one_at_the_budget_runs() {
    let frame = mosaic(1200, 800);
    let (edits, options) = (Edits::default(), preview());
    let bytes = estimate_render(&frame, &edits, &options).unwrap();
    match budgeted(bytes - 1).render(&frame, &edits, &options) {
        Err(PipelineError::OverBudget(message)) if message.contains("MiB of memory") => {}
        Err(e) => panic!("an over-budget render failed with the wrong error: {e}"),
        Ok(_) => panic!("a render over the memory budget ran"),
    }
    if let Err(e) = budgeted(bytes).render(&frame, &edits, &options) {
        panic!("a render at the memory budget was refused: {e}");
    }
}

#[test]
fn idle_sensor_stages_are_evicted_to_admit_a_render() {
    let frame = mosaic(1200, 800);
    let (edits, options) = (Edits::default(), preview());
    let budget = estimate_render(&frame, &edits, &options).unwrap();
    let renderer = budgeted(budget);
    renderer.render(&frame, &edits, &options).unwrap();
    if renderer.sensor_cache.retained_bytes() == 0 {
        panic!("the preview left no sensor stage in the cache");
    }
    let admitted = renderer.admit(budget, None);
    if admitted.is_err() || renderer.sensor_cache.retained_bytes() != 0 {
        panic!(
            "admitting a full-budget render kept {} cached bytes",
            renderer.sensor_cache.retained_bytes()
        );
    }
}

#[test]
fn a_cached_sensor_stage_is_admitted_without_its_sensor_work() {
    let frame = mosaic(2000, 1500);
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    let options = preview();
    let host = HostMemory::new(Some(estimate_render(&frame, &edits, &options).unwrap()));
    let warm = CpuRenderer::with_host(host.clone());
    warm.render(&frame, &edits, &options).unwrap();
    let _in_use = host.admit(1, None).unwrap();
    let token = CancelTracker::new().next();
    let (sent, received) = mpsc::channel();
    let rendered = thread::scope(|scope| {
        scope.spawn(|| {
            let result = warm.render_with_cancel(&frame, &edits, &options, Some(&token));
            sent.send(result.map(drop)).unwrap();
        });
        let result = received.recv_timeout(Duration::from_secs(60));
        token.cancel();
        result.unwrap_or_else(|_| received.recv().unwrap())
    });
    if let Err(e) = rendered {
        panic!("a render that hits the sensor cache was not admitted beside memory in use: {e}");
    }
}
