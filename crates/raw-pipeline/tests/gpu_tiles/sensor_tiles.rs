use raw_pipeline::edits::{Edits, MaskedEdits};
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::{GpuRenderer, GpuRendererOptions, TilePlan, TilingReport};
use raw_pipeline_testkit::denoise::denoise_plane;
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::gpu::try_renderer_with;
use raw_pipeline_testkit::masks::layer;
use raw_pipeline_testkit::render::rgb8_opts;

use super::{CANON_5D, Mode, Pair, XT2, decoded, radial, render, require_tiles};

const TILES: TilePlan = TilePlan::SensorTiled { edge: 1024 };
const DJI: &str = "DJI_FC7303_16-9.dng";

fn heavy_edits() -> Edits {
    let mut edits = Edits::default();
    edits.basic.dehaze = 25.0;
    edits.basic.clarity = 30.0;
    edits.basic.texture = 25.0;
    edits.tone.shadows = 40.0;
    edits.detail.luma_nr_amount = 40.0;
    edits.detail.color_nr_amount = 50.0;
    edits.detail.capture_corner_boost = 60.0;
    edits.effects.grain_amount = 20.0;
    edits.effects.vignette_amount = -30.0;
    edits.geometry.rotate_angle = 2.5;
    edits.lens.profile_enabled = Some(true);
    edits.lens.ca_enabled = true;
    edits.lens.k1 = -0.08;
    edits.lens.k2 = 0.02;
    edits.lens.vk1 = -0.4;
    edits.lens.ca_red_scale_x10000 = 6.0;
    edits.lens.ca_blue_scale_x10000 = -5.0;
    let clarity = MaskedEdits {
        exposure_ev: Some(0.6),
        clarity: Some(30.0),
        ..Default::default()
    };
    edits.masks = vec![layer("radial", radial((0.4, 0.55), (0.3, 0.25)), clarity)];
    edits
}

fn lens_only() -> Edits {
    let mut edits = Edits {
        lens: heavy_edits().lens,
        ..Edits::default()
    };
    edits.detail.capture_sharpen = false;
    edits
}

fn without_dehaze() -> Edits {
    let mut edits = heavy_edits();
    edits.basic.dehaze = 0.0;
    edits
}

fn export(max_edge: u32) -> RenderOptions {
    RenderOptions {
        quality: true,
        ..rgb8_opts(max_edge)
    }
}

fn modes(frame: &RawFrame) -> [Mode; 2] {
    let long_edge = frame.meta.width.max(frame.meta.height) as u32;
    [
        ("half export", export(long_edge / 2)),
        ("preview", rgb8_opts(2048)),
    ]
}

#[test]
fn sensor_tiles_match_the_single_pass_on_real_raws() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    for name in [XT2, DJI] {
        let frame = decoded(name);
        pair.require_modes(name, &frame, &heavy_edits(), &modes(&frame));
    }
    let frame = decoded(CANON_5D);
    let opts = export(frame.meta.width as u32);
    pair.require("5D export", &frame, &heavy_edits(), &opts);
}

#[test]
fn sensor_tiles_match_the_single_pass_on_the_fast_plan_and_without_dehaze() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let frame = decoded(CANON_5D);
    let [half, preview] = modes(&frame);
    pair.require_modes("5D fast plan", &frame, &lens_only(), &[preview]);
    pair.require_modes("5D without dehaze", &frame, &without_dehaze(), &[half]);
}

#[test]
fn sensor_tiles_match_the_single_pass_with_an_ai_denoise_blend() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let frame = decoded(CANON_5D);
    let (w, h) = (frame.meta.width, frame.meta.height);
    let rgb: Vec<f32> = (0..w * h)
        .flat_map(|i| [(i % w) as f32 / w as f32, (i / w) as f32 / h as f32, 0.3])
        .collect();
    let mut edits = Edits::default();
    edits.detail.ai_denoise = true;
    let modes = modes(&frame).map(|(mode, opts)| {
        let opts = RenderOptions {
            denoise: Some(denoise_plane(&frame, 7, &rgb)),
            ..opts
        };
        (mode, opts)
    });
    pair.require_modes("ai denoise", &frame, &edits, &modes);
}

fn render_over_budget(width: usize, height: usize, budget: u64) {
    let Some(renderer) = try_renderer_with(GpuRendererOptions {
        memory_budget_bytes: Some(budget),
        ..GpuRendererOptions::default()
    }) else {
        return;
    };
    let frame = synthetic_bayer_frame(width, height, "RGGB");
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    edits.detail.color_nr_amount = 50.0;
    let img = render(&renderer, &frame, &edits, &export(width as u32));
    if (img.width as usize, img.height as usize) != (width, height) {
        panic!("{width}x{height} rendered {}x{}", img.width, img.height);
    }
    if !matches!(img.tiling.plan, TilePlan::SensorTiled { .. }) {
        panic!(
            "{width}x{height} in {budget} bytes rendered as {:?}",
            img.tiling
        );
    }
}

#[test]
fn sensor_tiles_render_a_mosaic_over_the_single_pass_budget() {
    render_over_budget(6000, 4000, 1536 << 20);
}

#[test]
#[ignore = "renders a 100 MP frame; run by hand on a GPU with 8 GiB to spare"]
fn sensor_tiles_render_a_100_mp_mosaic_over_the_single_pass_budget() {
    render_over_budget(11648, 8736, 6 << 30);
}

#[test]
fn sensor_tiles_render_a_source_and_report_it() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let frame = decoded(CANON_5D);
    let opts = rgb8_opts(2048);
    let edits = lens_only();
    let source = |renderer: &GpuRenderer| match renderer.render_source(&frame, &edits, &opts, None)
    {
        Ok(source) => source,
        Err(e) => panic!("source render failed: {e}"),
    };
    let reference = source(&pair.single);
    let got = source(&pair.tiled);
    if reference.tiling != TilingReport::SINGLE_PASS {
        panic!("the single pass source reported {:?}", reference.tiling);
    }
    require_tiles("the tiled source", got.tiling, true);
    if got.image.rgb_f16 != reference.image.rgb_f16 {
        panic!("the tiled source differs from the single pass");
    }
}
