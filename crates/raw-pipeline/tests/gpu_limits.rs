use raw_pipeline::PipelineError;
use raw_pipeline::edits::Edits;
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::render::rgb8_opts;

#[test]
fn the_adapter_texture_limit_is_the_largest_frame_the_gpu_renders() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let limit = renderer.context().adapter.limits().max_texture_dimension_2d;
    let oversized = synthetic_bayer_frame(limit as usize + 1, 16, "RGGB");
    match renderer.render(&oversized, &Edits::default(), &rgb8_opts(limit + 1)) {
        Err(PipelineError::Unsupported(_)) => {}
        Err(e) => panic!("a frame one pixel past the limit failed with {e}"),
        Ok(img) => panic!(
            "a frame one pixel past the limit rendered at {}x{}",
            img.width, img.height
        ),
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
