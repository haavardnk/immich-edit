use raw_pipeline::decode;
use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    Vec2f,
};
use raw_pipeline::frame::{RawFrame, RenderOptions, RenderedImage};
use raw_pipeline::{GpuRenderer, GpuRendererOptions, TilePlan, TilingReport};
use raw_pipeline_testkit::denoise::denoise_plane;
use raw_pipeline_testkit::fixtures::fixture_path;
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::gpu::{try_renderer, try_renderer_with};
use raw_pipeline_testkit::render::rgb8_opts;

const TILE_EDGE: u32 = 256;
const FIXTURES: [&str; 3] = [
    "Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf",
    "Canon_EOS_5D_3-2.cr2",
    "DJI_FC7303_16-9.dng",
];

fn decoded(name: &str) -> RawFrame {
    let bytes = std::fs::read(fixture_path(name)).unwrap();
    decode::decode(&bytes).unwrap()
}

fn tiled_renderer() -> Option<GpuRenderer> {
    try_renderer_with(GpuRendererOptions {
        tile_edge: Some(TILE_EDGE),
        ..GpuRendererOptions::default()
    })
}

fn radial_mask() -> MaskLayer {
    MaskLayer {
        id: "radial".into(),
        name: String::new(),
        enabled: true,
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Radial {
                center: Vec2f { x: 0.4, y: 0.55 },
                radius_xy: Vec2f { x: 0.3, y: 0.25 },
                feather: 0.4,
                angle: 0.0,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            exposure_ev: Some(0.6),
            clarity: Some(30.0),
            ..Default::default()
        },
    }
}

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
    edits.masks.push(radial_mask());
    edits
}

fn render(
    renderer: &GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> RenderedImage {
    match renderer.render(frame, edits, opts) {
        Ok(img) => img,
        Err(e) => panic!("render failed: {e}"),
    }
}

fn require_identical(label: &str, single: &RenderedImage, tiled: &RenderedImage) {
    if (single.width, single.height) != (tiled.width, tiled.height) {
        panic!(
            "{label}: tiled {}x{} against single {}x{}",
            tiled.width, tiled.height, single.width, single.height
        );
    }
    let differing = single
        .bytes
        .iter()
        .zip(&tiled.bytes)
        .filter(|(a, b)| a != b)
        .count();
    if differing > 0 {
        panic!("{label}: {differing} tiled bytes differ from the single pass");
    }
}

fn modes(frame: &RawFrame) -> [(&'static str, RenderOptions); 2] {
    let long_edge = frame.meta.width.max(frame.meta.height) as u32;
    [
        (
            "export",
            RenderOptions {
                quality: true,
                ..rgb8_opts(long_edge)
            },
        ),
        ("preview", rgb8_opts(1200)),
    ]
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

fn require_tiled_parity(single: &GpuRenderer, tiled: &GpuRenderer, name: &str, edits: &Edits) {
    let frame = decoded(name);
    for (mode, opts) in modes(&frame) {
        let reference = render(single, &frame, edits, &opts);
        let got = render(tiled, &frame, edits, &opts);
        require_identical(&format!("{name} {mode}"), &reference, &got);
        if !matches!(got.tiling.plan, TilePlan::SensorTiled { .. }) || got.tiling.tiles < 4 {
            panic!("{name} {mode} rendered as {:?}", got.tiling);
        }
    }
}

#[test]
fn tiled_renders_match_single_pass_on_real_raws() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    for name in FIXTURES {
        require_tiled_parity(&single, &tiled, name, &heavy_edits());
    }
}

#[test]
fn tiled_fast_and_dehaze_free_renders_match_single_pass() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    for edits in [lens_only(), without_dehaze()] {
        require_tiled_parity(&single, &tiled, "Canon_EOS_5D_3-2.cr2", &edits);
    }
}

#[test]
fn a_tiled_ai_denoise_blend_matches_single_pass() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    let frame = decoded("Canon_EOS_5D_3-2.cr2");
    let (w, h) = (frame.meta.width, frame.meta.height);
    let rgb: Vec<f32> = (0..w * h)
        .flat_map(|i| [(i % w) as f32 / w as f32, (i / w) as f32 / h as f32, 0.3])
        .collect();
    let mut edits = Edits::default();
    edits.detail.ai_denoise = true;
    for (mode, opts) in modes(&frame) {
        let opts = RenderOptions {
            denoise: Some(denoise_plane(&frame, 7, &rgb)),
            ..opts
        };
        let reference = render(&single, &frame, &edits, &opts);
        let got = render(&tiled, &frame, &edits, &opts);
        require_identical(&format!("ai denoise {mode}"), &reference, &got);
    }
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
    let opts = RenderOptions {
        quality: true,
        ..rgb8_opts(width as u32)
    };
    let img = render(&renderer, &frame, &edits, &opts);
    if (img.width as usize, img.height as usize) != (width, height) {
        panic!("{width}x{height} rendered {}x{}", img.width, img.height);
    }
    if !matches!(img.tiling.plan, TilePlan::SensorTiled { .. }) {
        panic!(
            "a {width}x{height} render inside a {budget} byte budget rendered as {:?}",
            img.tiling
        );
    }
}

#[test]
fn a_mosaic_over_the_single_pass_budget_renders_in_tiles() {
    render_over_budget(6000, 4000, 1536 << 20);
}

#[test]
#[ignore = "renders a 100 MP frame; run by hand on a GPU with 8 GiB to spare"]
fn a_100_mp_mosaic_over_the_single_pass_budget_renders_in_tiles() {
    render_over_budget(11648, 8736, 6 << 30);
}

#[test]
fn a_source_rendered_in_sensor_tiles_reports_them() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    let frame = decoded("Canon_EOS_5D_3-2.cr2");
    let opts = rgb8_opts(1200);
    let edits = lens_only();
    let source = |renderer: &GpuRenderer| match renderer.render_source(&frame, &edits, &opts, None)
    {
        Ok(source) => source,
        Err(e) => panic!("source render failed: {e}"),
    };
    let reference = source(&single);
    let got = source(&tiled);
    if reference.tiling != TilingReport::SINGLE_PASS {
        panic!("the single pass source reported {:?}", reference.tiling);
    }
    if !matches!(got.tiling.plan, TilePlan::SensorTiled { .. }) || got.tiling.tiles < 4 {
        panic!("the tiled source reported {:?}", got.tiling);
    }
    if got.image.rgb_f16 != reference.image.rgb_f16 {
        panic!("the tiled source differs from the single pass");
    }
}
