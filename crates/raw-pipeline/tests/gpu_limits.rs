use raw_pipeline::edits::Edits;
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::render::rgb8_opts;

#[test]
fn gpu_renders_a_frame_as_wide_as_the_texture_limit() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let limit = renderer.context().adapter.limits().max_texture_dimension_2d;
    let width = limit as usize;
    let frame = synthetic_bayer_frame(width, 16, "RGGB");
    let img = renderer
        .render(&frame, &Edits::default(), &rgb8_opts(limit))
        .unwrap();
    if (img.width, img.height) != (limit, 16) {
        panic!(
            "frame {limit}x16 at the texture limit rendered at {}x{}",
            img.width, img.height
        );
    }
}
