use std::time::{Duration, Instant};

use raw_pipeline::edits::Edits;
use raw_pipeline::frame::{RawFrame, RenderOptions, RenderedImage};
use raw_pipeline::gpu::RenderPlan;
use raw_pipeline::{CpuRenderer, GpuRenderer, GpuRendererOptions, decode};

mod common;

const FIXTURE: &str = "Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf";
const ITERS: usize = 3;
const EDGES: [u32; 2] = [1600, 3000];

struct Stage {
    name: &'static str,
    wall: Vec<Duration>,
    gpu: Vec<Duration>,
}

fn fast_edits() -> Edits {
    let mut edits = Edits::default();
    edits.basic.exposure_ev = 0.3;
    edits.basic.contrast = 15.0;
    edits.detail.capture_sharpen = false;
    edits
}

fn presence_edits() -> Edits {
    let mut edits = fast_edits();
    edits.basic.clarity = 20.0;
    edits.basic.dehaze = 10.0;
    edits.tone.shadows = 20.0;
    edits
}

fn median(samples: &[Duration]) -> Option<Duration> {
    let mut sorted = samples.to_vec();
    sorted.sort();
    sorted.get(sorted.len() / 2).copied()
}

fn ms(d: Option<Duration>) -> String {
    d.map_or_else(
        || "—".into(),
        |d| format!("{:.1}", d.as_secs_f64() * 1000.0),
    )
}

fn report(label: &str, render: impl Fn(&Edits, &RenderOptions) -> RenderedImage, frame: &RawFrame) {
    for (plan, edits) in [("Fast", fast_edits()), ("Presence", presence_edits())] {
        let selected = RenderPlan::select(&edits, frame);
        let expected = if plan == "Fast" {
            RenderPlan::Fast
        } else {
            RenderPlan::Presence
        };
        if selected != expected {
            panic!("{plan} edits select the {selected:?} plan");
        }
        for max_edge in EDGES {
            let options = RenderOptions {
                max_edge,
                histogram: true,
                ..Default::default()
            };
            let started = Instant::now();
            render(&edits, &options);
            let first = started.elapsed();
            let mut totals: Vec<Duration> = Vec::with_capacity(ITERS);
            let mut stages: Vec<Stage> = Vec::new();
            for _ in 0..ITERS {
                let started = Instant::now();
                let image = render(&edits, &options);
                totals.push(started.elapsed());
                for t in &image.timings {
                    let index = match stages.iter().position(|s| s.name == t.stage) {
                        Some(i) => i,
                        None => {
                            stages.push(Stage {
                                name: t.stage,
                                wall: Vec::new(),
                                gpu: Vec::new(),
                            });
                            stages.len() - 1
                        }
                    };
                    stages[index].wall.push(t.wall);
                    stages[index].gpu.extend(t.gpu);
                }
            }
            println!(
                "\n#### {label} {max_edge} px {plan}: {} ms per tick, first render {} ms\n",
                ms(median(&totals)),
                ms(Some(first))
            );
            println!("| Stage | Wall ms | GPU ms |\n| --- | --- | --- |");
            for s in &stages {
                println!(
                    "| {} | {} | {} |",
                    s.name,
                    ms(median(&s.wall)),
                    ms(median(&s.gpu))
                );
            }
        }
    }
}

#[test]
fn stage_report() {
    if std::env::var_os("STAGE_REPORT").is_none() {
        eprintln!("skip: set STAGE_REPORT=1 to print per-stage render timings");
        return;
    }
    let path = common::fixture_path(FIXTURE);
    let frame = decode::decode(&std::fs::read(&path).unwrap()).unwrap();
    println!("{FIXTURE}: {}x{}", frame.width, frame.height);

    let cpu = CpuRenderer::new();
    report(
        "cpu",
        |edits, options| cpu.render(&frame, edits, options).unwrap(),
        &frame,
    );

    let renderer = match GpuRenderer::with_options(GpuRendererOptions {
        timestamps: true,
        ..Default::default()
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no gpu adapter, cpu only: {e}");
            return;
        }
    };
    let label = if renderer.gpu_timestamps() {
        "gpu"
    } else {
        "gpu (no timestamp queries)"
    };
    report(
        label,
        |edits, options| renderer.render(&frame, edits, options).unwrap(),
        &frame,
    );
}
