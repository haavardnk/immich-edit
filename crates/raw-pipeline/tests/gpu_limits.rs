use raw_pipeline::cpu::estimate_render;
use raw_pipeline::edits::Edits;
use raw_pipeline::encode::working_bytes;
use raw_pipeline::frame::{OutputFormat, RenderOptions};
use raw_pipeline::gpu::RenderSource;
use raw_pipeline::{
    CpuRenderer, GpuRenderer, GpuRendererOptions, HostMemory, PipelineError, TilePlan,
};
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
    expected: fn(&PipelineError) -> bool,
) {
    match result {
        Err(e) if expected(&e) && !renderer.is_lost() => {}
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
        |e| matches!(e, PipelineError::Unsupported(_)),
    );
    match renderer.render(&oversized, &shadows(), &opts) {
        Ok(img)
            if (img.width, img.height) == (limit + 1, 16)
                && matches!(img.tiling.plan, TilePlan::DisplayTiled { .. }) => {}
        Ok(img) => panic!(
            "a frame one pixel past the limit rendered {}x{} as {:?}",
            img.width, img.height, img.tiling
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
        |e| matches!(e, PipelineError::OverBudget(_)),
    );
    let small = synthetic_bayer_frame(256, 256, "RGGB");
    if let Err(e) = renderer.render(&small, &Edits::default(), &rgb8_opts(256)) {
        panic!("a small frame after a refused one failed: {e}");
    }
    if renderer.memory().budget_bytes != Some(BUDGET) {
        panic!("the budget became {:?}", renderer.memory().budget_bytes);
    }
}

#[test]
fn a_gpu_render_admits_its_readback_and_encode_against_host_memory() {
    let frame = synthetic_bayer_frame(4000, 3000, "RGGB");
    let edits = Edits::default();
    let options = RenderOptions {
        max_edge: 4000,
        output: OutputFormat::Webp {
            quality: 90,
            lossless: true,
        },
        ..RenderOptions::default()
    };
    let with_host = |budget: u64| {
        try_renderer_with(GpuRendererOptions {
            memory_budget_bytes: Some(u64::MAX),
            host: HostMemory::new(Some(budget)),
            ..GpuRendererOptions::default()
        })
    };
    let Some(probe) = with_host(u64::MAX) else {
        return;
    };
    let need = probe.estimate_render(
        RenderSource::Raw(&frame),
        &edits,
        &options,
        TilePlan::SinglePass,
    );
    if need.host < working_bytes(&options.output, (4000, 3000)) {
        panic!("the host estimate {} leaves out the encoder", need.host);
    }
    let unified = probe.memory().device.is_some_and(|device| device.unified);
    let single = need.host + if unified { need.device } else { 0 };
    let tight = with_host(single - 1).unwrap();
    match tight.render(&frame, &edits, &options) {
        Ok(img) if unified && img.tiling.plan != TilePlan::SinglePass => {}
        Err(PipelineError::OverBudget(message))
            if !unified && message.contains("MiB of memory") => {}
        Ok(img) => panic!("a render past the host budget ran as {:?}", img.tiling),
        Err(e) => panic!("a render past the host budget failed with {e}"),
    }
    match with_host(single).unwrap().render(&frame, &edits, &options) {
        Ok(img) if img.tiling.plan == TilePlan::SinglePass => {}
        Ok(img) => panic!("a render at the host budget ran as {:?}", img.tiling),
        Err(e) => panic!("a render at the host budget failed with {e}"),
    }
    let starved = with_host(1 << 20).unwrap();
    require_refused(
        &starved,
        "a render past any plan's host memory",
        starved.render(&frame, &edits, &options),
        |e| matches!(e, PipelineError::OverBudget(message) if message.contains("MiB of memory")),
    );
}

#[test]
fn a_cpu_render_trims_idle_gpu_caches_only_on_unified_memory() {
    let frame = synthetic_bayer_frame(2000, 1500, "RGGB");
    let (edits, options) = (shadows(), rgb8_opts(2000));
    let budget = estimate_render(&frame, &edits, &options).unwrap();
    let host = HostMemory::new(Some(budget));
    let Some(gpu) = try_renderer_with(GpuRendererOptions {
        memory_budget_bytes: Some(u64::MAX),
        host: host.clone(),
        ..GpuRendererOptions::default()
    }) else {
        return;
    };
    gpu.render(&frame, &edits, &options).unwrap();
    let cached = |gpu: &GpuRenderer| {
        let stats = gpu.pool_stats();
        stats.texture_pool + stats.output_targets
    };
    let before = cached(&gpu);
    if before == 0 {
        panic!("the gpu render left nothing in its pools");
    }
    CpuRenderer::with_host(host)
        .render(&frame, &edits, &options)
        .unwrap();
    let unified = gpu.memory().device.is_some_and(|device| device.unified);
    let expected = if unified { 0 } else { before };
    if cached(&gpu) != expected {
        panic!(
            "a cpu render left {} of {before} gpu pool bytes, unified={unified}",
            cached(&gpu)
        );
    }
}
