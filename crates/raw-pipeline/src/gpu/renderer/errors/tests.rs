use wgpu::{TextureFormat, TextureUsages};

use super::super::test_frame::checker_frame;
use super::GpuRenderer;
use crate::edits::Edits;
use crate::frame::{OutputFormat, RenderOptions};
use crate::gpu::texture::texture_2d;
use crate::{PipelineError, PipelineResult};

fn oversized_texture(renderer: &GpuRenderer) {
    let edge = renderer.ctx.device.limits().max_texture_dimension_2d + 1;
    texture_2d(
        &renderer.ctx.device,
        "past-the-limit",
        TextureFormat::Rgba8Unorm,
        (edge, 1),
        1,
        TextureUsages::TEXTURE_BINDING,
    );
}

fn render(renderer: &GpuRenderer) -> PipelineResult<()> {
    let opts = RenderOptions {
        output: OutputFormat::Rgb8,
        ..RenderOptions::default()
    };
    let mut edits = Edits::default();
    edits.basic.dehaze = 0.3;
    renderer.render(&checker_frame(), &edits, &opts).map(drop)
}

#[test]
fn a_gpu_error_is_returned_and_discards_every_pooled_resource() {
    let Ok(renderer) = GpuRenderer::new() else {
        return;
    };
    if let Err(e) = render(&renderer) {
        panic!("warm-up render failed: {e}");
    }
    let warm = renderer.pool_stats();
    if warm.output_targets == 0
        || warm.uniform_pool == 0
        || renderer.sensor.frames.lock().is_empty()
        || renderer.sensor.atmospheres.lock().is_empty()
    {
        panic!("the warm-up render pooled nothing to discard: {warm:?}");
    }
    let failed = pollster::block_on(renderer.guarded(async {
        oversized_texture(&renderer);
        Ok(())
    }));
    if !matches!(failed, Err(PipelineError::Render(_))) {
        panic!("an oversized texture inside the guard returned {failed:?}");
    }
    let after = renderer.pool_stats();
    let held = [
        after.texture_pool,
        after.uniform_pool,
        after.output_targets,
        after.sharpen_targets,
        after.wb_cache,
        after.nr_cache,
        after.capture_cache,
        after.denoise_cache,
        after.atlas_pool,
    ];
    if held.iter().any(|bytes| *bytes != 0)
        || !renderer.sensor.frames.lock().is_empty()
        || !renderer.sensor.atmospheres.lock().is_empty()
    {
        panic!("a captured gpu error left pooled resources behind: {after:?}");
    }
    if let Err(e) = render(&renderer) {
        panic!("the render after a discard failed: {e}");
    }
}

#[test]
fn a_cancelled_render_stays_cancelled_when_the_gpu_also_failed() {
    let Ok(renderer) = GpuRenderer::new() else {
        return;
    };
    let result: PipelineResult<()> = pollster::block_on(renderer.guarded(async {
        oversized_texture(&renderer);
        Err(PipelineError::Cancelled)
    }));
    if !matches!(result, Err(PipelineError::Cancelled)) {
        panic!("a cancelled render with a gpu error returned {result:?}");
    }
}

#[test]
fn an_error_outside_the_guard_is_recorded_not_panicked() {
    let Ok(renderer) = GpuRenderer::new() else {
        return;
    };
    oversized_texture(&renderer);
    if renderer.ctx.take_uncaptured_error().is_none() {
        panic!("an uncaptured error was not recorded");
    }
    if renderer.ctx.take_uncaptured_error().is_some() {
        panic!("a recorded error was not cleared by take");
    }
}
