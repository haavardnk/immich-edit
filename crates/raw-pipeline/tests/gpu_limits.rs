use raw_pipeline::edits::Edits;
use raw_pipeline::{GpuRenderer, GpuRendererOptions, PipelineError};
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::gpu::{try_renderer, try_renderer_with};
use raw_pipeline_testkit::render::rgb8_opts;
use raw_pipeline_testkit::retouch::untileable_edits;

fn shadows() -> Edits {
    let mut edits = Edits::default();
    edits.tone.shadows = 40.0;
    edits
}

fn require_refused(
    renderer: &GpuRenderer,
    what: &str,
    result: raw_pipeline::PipelineResult<raw_pipeline::frame::RenderedImage>,
) {
    match result {
        Err(PipelineError::Unsupported(_)) if !renderer.is_lost() => {}
        Err(e) => panic!("{what} failed with {e}"),
        Ok(img) => panic!("{what} rendered at {}x{}", img.width, img.height),
    }
}

#[test]
fn the_adapter_texture_limit_is_the_largest_frame_the_gpu_renders() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let limit = renderer.context().adapter.limits().max_texture_dimension_2d;
    let oversized = synthetic_bayer_frame(limit as usize + 1, 16, "RGGB");
    let opts = rgb8_opts(limit + 1);
    let result = renderer.render(&oversized, &untileable_edits(), &opts);
    require_refused(
        &renderer,
        "an untileable frame one pixel past the limit",
        result,
    );
    match renderer.render(&oversized, &shadows(), &opts) {
        Ok(img) if (img.width, img.height) == (limit + 1, 16) && renderer.display_tiles() > 0 => {}
        Ok(img) => panic!(
            "a frame one pixel past the limit rendered {}x{} in {} display tiles",
            img.width,
            img.height,
            renderer.display_tiles()
        ),
        Err(e) => panic!("a frame one pixel past the limit failed with {e}"),
    }
    let frame = synthetic_bayer_frame(limit as usize, 16, "RGGB");
    let img = match renderer.render(&frame, &Edits::default(), &rgb8_opts(limit)) {
        Ok(img) => img,
        Err(e) => panic!("a frame at the {limit} px limit failed after an oversized one: {e}"),
    };
    if (img.width, img.height) != (limit, 16) || renderer.is_lost() {
        panic!(
            "frame {limit}x16 at the texture limit rendered at {}x{}, lost={}",
            img.width,
            img.height,
            renderer.is_lost()
        );
    }
}

#[test]
fn a_preview_of_a_frame_past_the_limit_renders_from_superpixels() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let limit = renderer.context().device.limits().max_texture_dimension_2d;
    let frame = synthetic_bayer_frame(2 * limit as usize, 512, "RGGB");
    let img = match renderer.render(&frame, &Edits::default(), &rgb8_opts(1024)) {
        Ok(img) => img,
        Err(e) => panic!(
            "a superpixel preview of a {}x512 frame failed: {e}",
            2 * limit
        ),
    };
    if img.width != 1024 || img.renderer != "gpu" {
        panic!(
            "superpixel preview rendered {}x{} on {}",
            img.width, img.height, img.renderer
        );
    }
}

#[test]
fn a_preview_of_the_widest_export_renders_from_larger_superpixels() {
    const WIDEST: usize = 65535;
    let Some(renderer) = try_renderer() else {
        return;
    };
    if renderer.context().device.limits().max_texture_dimension_2d < 16384 {
        return;
    }
    let frame = synthetic_bayer_frame(WIDEST, 512, "RGGB");
    let img = match renderer.render(&frame, &Edits::default(), &rgb8_opts(1024)) {
        Ok(img) => img,
        Err(e) => panic!("a superpixel preview of a {WIDEST}x512 frame failed: {e}"),
    };
    if img.width != 1024 || img.renderer != "gpu" {
        panic!(
            "superpixel preview rendered {}x{} on {}",
            img.width, img.height, img.renderer
        );
    }
}

#[test]
fn a_frame_over_the_memory_budget_is_refused_and_a_small_one_renders() {
    const BUDGET: u64 = 64 << 20;
    let Some(renderer) = try_renderer_with(GpuRendererOptions {
        memory_budget_bytes: Some(BUDGET),
        ..GpuRendererOptions::default()
    }) else {
        return;
    };
    let large = synthetic_bayer_frame(4000, 3000, "RGGB");
    let result = renderer.render(&large, &untileable_edits(), &rgb8_opts(4000));
    require_refused(
        &renderer,
        "an untileable frame over the memory budget",
        result,
    );
    let small = synthetic_bayer_frame(256, 256, "RGGB");
    if let Err(e) = renderer.render(&small, &Edits::default(), &rgb8_opts(256)) {
        panic!("a small frame after a refused one failed: {e}");
    }
    if renderer.memory().budget_bytes != Some(BUDGET) {
        panic!("the budget became {:?}", renderer.memory().budget_bytes);
    }
}
