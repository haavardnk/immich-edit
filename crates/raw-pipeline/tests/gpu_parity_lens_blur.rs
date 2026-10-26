use raw_pipeline::GpuRenderer;
use raw_pipeline::edits::{BokehShape, Edits};
use raw_pipeline::frame::{RawFrame, RenderOptions, RenderedImage};
use raw_pipeline_testkit::depth::{bokeh_frame, depth_rasters, lens_blur_edits};
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::parity::{ParityLedger, mean_abs_delta, require_same_dims};
use raw_pipeline_testkit::render::rgb8_opts;

fn opts(max_edge: u32) -> RenderOptions {
    RenderOptions {
        rasters: depth_rasters(),
        ..rgb8_opts(max_edge)
    }
}

fn require_effect(label: &str, sharp: &RenderedImage, blurred: &RenderedImage) {
    let delta = mean_abs_delta(&sharp.bytes, &blurred.bytes);
    if delta < 1.0 {
        panic!("{label}: lens blur moved the render by only {delta:.3}");
    }
}

fn check_parity(
    renderer: &GpuRenderer,
    ledger: &mut ParityLedger,
    label: &str,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
    limit: f64,
) {
    let gpu = renderer.render(frame, edits, opts).unwrap();
    let cpu = raw_pipeline::cpu::render(frame, edits, opts).unwrap();
    require_same_dims(label, &cpu, &gpu);
    let mut sharp = edits.clone();
    sharp.lens_blur.amount = 0.0;
    let cpu_sharp = raw_pipeline::cpu::render(frame, &sharp, opts).unwrap();
    let gpu_sharp = renderer.render(frame, &sharp, opts).unwrap();
    require_effect(&format!("{label} cpu"), &cpu_sharp, &cpu);
    require_effect(&format!("{label} gpu"), &gpu_sharp, &gpu);
    ledger.check(label, &cpu.bytes, &gpu.bytes, limit);
}

#[test]
fn gpu_lens_blur_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let mut ledger = ParityLedger::new("lens_blur");
    let cases = [
        ("circle", 100.0, BokehShape::Circle, 0.0),
        ("hexagon-boost", 70.0, BokehShape::Hexagon, 80.0),
        ("octagon", 100.0, BokehShape::Octagon, 30.0),
    ];
    for (w, h) in [(160, 120), (961, 641)] {
        let frame = bokeh_frame(w, h);
        for (name, amount, shape, boost) in cases {
            let edits = lens_blur_edits(amount, shape, boost);
            let label = format!("{name}-{w}x{h}");
            check_parity(
                &renderer,
                &mut ledger,
                &label,
                &frame,
                &edits,
                &opts(w as u32),
                0.1,
            );
        }
    }
    ledger.finish();
}

#[test]
fn gpu_lens_blur_matches_cpu_in_every_orientation() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let mut ledger = ParityLedger::new("lens_blur");
    let mut edits = lens_blur_edits(100.0, BokehShape::Hexagon, 40.0);
    edits.lens.k1 = -0.15;
    edits.lens.k2 = 0.05;
    for transpose in [false, true] {
        for flip_x in [false, true] {
            for flip_y in [false, true] {
                let mut frame = bokeh_frame(903, 601);
                frame.meta.orientation = (transpose, flip_x, flip_y);
                let label = format!("orientation-{transpose}-{flip_x}-{flip_y}");
                check_parity(
                    &renderer,
                    &mut ledger,
                    &label,
                    &frame,
                    &edits,
                    &opts(903),
                    0.1,
                );
            }
        }
    }
    ledger.finish();
}

#[test]
fn gpu_lens_blur_follows_dehaze_at_preview_size() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let mut ledger = ParityLedger::new("lens_blur");
    let mut edits = lens_blur_edits(80.0, BokehShape::Circle, 50.0);
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 20.0;
    let frame = bokeh_frame(1600, 1067);
    check_parity(
        &renderer,
        &mut ledger,
        "dehaze-preview",
        &frame,
        &edits,
        &opts(700),
        0.3,
    );
    ledger.finish();
}

#[test]
fn a_missing_depth_map_renders_sharp_on_both_renderers() {
    let frame = bokeh_frame(160, 120);
    let mut edits = lens_blur_edits(100.0, BokehShape::Circle, 0.0);
    edits.lens_blur.depth_raster_id = "gone".into();
    let mut sharp = edits.clone();
    sharp.lens_blur.amount = 0.0;
    let opts = opts(160);
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_sharp = raw_pipeline::cpu::render(&frame, &sharp, &opts).unwrap();
    if cpu.bytes != cpu_sharp.bytes {
        panic!("cpu blurred without a depth map");
    }
    let Some(renderer) = try_renderer() else {
        return;
    };
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_sharp = renderer.render(&frame, &sharp, &opts).unwrap();
    let delta = mean_abs_delta(&gpu.bytes, &gpu_sharp.bytes);
    if delta > 0.5 {
        panic!("gpu blurred without a depth map: {delta:.3}");
    }
}
