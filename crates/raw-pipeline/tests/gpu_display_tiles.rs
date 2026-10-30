use raw_pipeline::decode;
use raw_pipeline::edits::{
    BokehShape, CropRect, Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer,
    MaskSource, MaskedEdits, Vec2f,
};
use raw_pipeline::frame::{
    BitDepth, OutputFormat, PngCompression, RawFrame, RenderOptions, RenderedImage,
};
use raw_pipeline::{GpuRenderer, GpuRendererOptions};
use raw_pipeline_testkit::depth::{bokeh_frame, depth_rasters, lens_blur_edits};
use raw_pipeline_testkit::fixtures::fixture_path;
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::gpu::{try_renderer, try_renderer_with};
use raw_pipeline_testkit::render::rgb8_opts;

const TILE_EDGE: u32 = 512;
const FIXTURES: [&str; 2] = [
    "Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf",
    "Canon_EOS_5D_3-2.cr2",
];

fn decoded(name: &str) -> RawFrame {
    let bytes = std::fs::read(fixture_path(name)).unwrap();
    decode::decode(&bytes).unwrap()
}

fn tiled_renderer() -> Option<GpuRenderer> {
    try_renderer_with(GpuRendererOptions {
        display_tile_edge: Some(TILE_EDGE),
        ..GpuRendererOptions::default()
    })
}

fn layer(id: &str, kind: MaskComponentKind, exposure_ev: f64) -> MaskLayer {
    MaskLayer {
        id: id.into(),
        name: String::new(),
        enabled: true,
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind,
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            exposure_ev: Some(exposure_ev),
            ..Default::default()
        },
    }
}

fn masks() -> Vec<MaskLayer> {
    let radial = MaskComponentKind::Radial {
        center: Vec2f { x: 0.4, y: 0.55 },
        radius_xy: Vec2f { x: 0.3, y: 0.25 },
        feather: 0.4,
        angle: 0.0,
    };
    let range = MaskComponentKind::LumaRange {
        min: 0.4,
        max: 1.0,
        softness: 0.05,
    };
    vec![layer("radial", radial, 0.6), layer("range", range, -0.8)]
}

fn local_edits() -> Edits {
    let mut edits = Edits::default();
    edits.basic.exposure_ev = 0.3;
    edits.detail.luma_nr_amount = 40.0;
    edits.detail.color_nr_amount = 50.0;
    edits.detail.capture_corner_boost = 60.0;
    edits.effects.grain_amount = 20.0;
    edits.effects.vignette_amount = -30.0;
    edits.lens.profile_enabled = Some(true);
    edits.lens.ca_enabled = true;
    edits.lens.k1 = -0.08;
    edits.lens.vk1 = -0.4;
    edits.lens.ca_red_scale_x10000 = 6.0;
    edits.masks = masks();
    edits
}

fn warped_edits() -> Edits {
    let mut edits = local_edits();
    edits.geometry.rotate_angle = 2.5;
    edits.geometry.crop = Some(CropRect {
        x: 0.1,
        y: 0.05,
        w: 0.8,
        h: 0.85,
    });
    edits
}

fn tight_crop_edits() -> Edits {
    let mut edits = local_edits();
    edits.geometry.crop = Some(CropRect {
        x: 0.2,
        y: 0.15,
        w: 0.6,
        h: 0.6,
    });
    edits
}

fn modes(frame: &RawFrame) -> [(&'static str, RenderOptions); 4] {
    let long_edge = frame.meta.width.max(frame.meta.height) as u32;
    let export = |max_edge| RenderOptions {
        quality: true,
        ..rgb8_opts(max_edge)
    };
    [
        ("export", export(long_edge)),
        ("half export", export(long_edge / 2)),
        ("preview", rgb8_opts(1600)),
        (
            "16-bit export",
            RenderOptions {
                output: OutputFormat::Png {
                    bit_depth: BitDepth::Sixteen,
                    compression: PngCompression::Fast,
                },
                ..export(long_edge)
            },
        ),
    ]
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
    if differing > 0 || single.bytes.len() != tiled.bytes.len() {
        panic!("{label}: {differing} tiled bytes differ from the single pass");
    }
}

fn require_tiled_parity(single: &GpuRenderer, tiled: &GpuRenderer, name: &str, edits: &Edits) {
    let frame = decoded(name);
    for (mode, opts) in modes(&frame) {
        let before = tiled.display_tiles();
        let reference = render(single, &frame, edits, &opts);
        let got = render(tiled, &frame, edits, &opts);
        require_identical(&format!("{name} {mode}"), &reference, &got);
        if tiled.display_tiles() - before < 4 {
            panic!("{name} {mode} rendered without display tiles");
        }
    }
}

#[test]
fn display_tiles_match_the_single_pass_on_real_raws() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    for name in FIXTURES {
        require_tiled_parity(&single, &tiled, name, &warped_edits());
    }
}

#[test]
fn display_tiles_without_a_geometry_warp_match_the_single_pass() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    require_tiled_parity(&single, &tiled, "Canon_EOS_5D_3-2.cr2", &local_edits());
}

#[test]
fn magnified_and_roi_previews_with_a_range_mask_match_the_single_pass() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    let roi = || RenderOptions {
        roi: Some(CropRect {
            x: 0.3,
            y: 0.25,
            w: 0.4,
            h: 0.45,
        }),
        ..rgb8_opts(1600)
    };
    let cases = [
        ("magnified preview", tight_crop_edits(), rgb8_opts(1600)),
        ("roi preview", local_edits(), roi()),
        ("context roi preview", context_edits(), roi()),
    ];
    for name in FIXTURES {
        let frame = decoded(name);
        for (mode, edits, opts) in &cases {
            let reference = render(&single, &frame, edits, opts);
            let got = render(&tiled, &frame, edits, opts);
            require_identical(&format!("{name} {mode}"), &reference, &got);
        }
    }
}

fn masked_presence(id: &str, edits: MaskedEdits) -> MaskLayer {
    let radial = MaskComponentKind::Radial {
        center: Vec2f { x: 0.6, y: 0.4 },
        radius_xy: Vec2f { x: 0.25, y: 0.3 },
        feather: 0.5,
        angle: 0.0,
    };
    MaskLayer {
        edits,
        ..layer(id, radial, 0.0)
    }
}

fn context_edits() -> Edits {
    let mut edits = warped_edits();
    edits.basic.dehaze = 35.0;
    edits.basic.texture = 40.0;
    edits.basic.clarity = 30.0;
    edits.tone.shadows = 45.0;
    edits.masks.push(masked_presence(
        "detail",
        MaskedEdits {
            texture: Some(-50.0),
            clarity: Some(60.0),
            shadows: Some(-30.0),
            ..Default::default()
        },
    ));
    edits
}

fn base_level_edits() -> Edits {
    let mut edits = local_edits();
    edits.basic.dehaze = -25.0;
    edits.tone.shadows = 40.0;
    edits.masks.push(masked_presence(
        "texture",
        MaskedEdits {
            texture: Some(70.0),
            ..Default::default()
        },
    ));
    edits
}

fn masked_shadows_edits() -> Edits {
    let mut edits = local_edits();
    edits.masks.push(masked_presence(
        "shadows",
        MaskedEdits {
            shadows: Some(60.0),
            ..Default::default()
        },
    ));
    edits
}

#[test]
fn display_tiles_with_a_shared_context_match_the_single_pass() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    for name in FIXTURES {
        require_tiled_parity(&single, &tiled, name, &context_edits());
    }
    for edits in [base_level_edits(), masked_shadows_edits()] {
        require_tiled_parity(&single, &tiled, "Canon_EOS_5D_3-2.cr2", &edits);
    }
}

#[test]
fn display_tiles_with_lens_blur_and_a_context_match_the_single_pass() {
    let (Some(single), Some(tiled)) = (try_renderer(), tiled_renderer()) else {
        return;
    };
    let opts = RenderOptions {
        rasters: depth_rasters(),
        quality: true,
        ..rgb8_opts(1800)
    };
    let mut edits = lens_blur_edits(80.0, BokehShape::Hexagon, 40.0);
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 40.0;
    edits.tone.shadows = 35.0;
    for orientation in [(false, false, false), (true, true, false)] {
        let mut frame = bokeh_frame(1800, 1200);
        frame.meta.orientation = orientation;
        let before = tiled.display_tiles();
        let reference = render(&single, &frame, &edits, &opts);
        let got = render(&tiled, &frame, &edits, &opts);
        if tiled.display_tiles() - before < 4 {
            panic!("the lens blur render {orientation:?} did not use display tiles");
        }
        require_identical(&format!("lens blur {orientation:?}"), &reference, &got);
    }
}

fn budget_edits() -> [(&'static str, Edits); 3] {
    let mut plain = Edits::default();
    plain.detail.luma_nr_amount = 40.0;
    let mut effects = plain.clone();
    effects.effects.grain_amount = 20.0;
    effects.effects.vignette_amount = -30.0;
    let mut spatial = plain.clone();
    spatial.basic.dehaze = 30.0;
    spatial.basic.clarity = 30.0;
    spatial.tone.shadows = 40.0;
    [("plain", plain), ("effects", effects), ("spatial", spatial)]
}

#[test]
fn a_render_over_the_sensor_tile_budget_renders_in_display_tiles() {
    let (Some(single), Some(budgeted)) = (
        try_renderer(),
        try_renderer_with(GpuRendererOptions {
            memory_budget_bytes: Some(320 << 20),
            ..GpuRendererOptions::default()
        }),
    ) else {
        return;
    };
    let frame = synthetic_bayer_frame(6000, 4000, "RGGB");
    let opts = RenderOptions {
        quality: true,
        ..rgb8_opts(6000)
    };
    for (label, edits) in budget_edits() {
        let before = budgeted.display_tiles();
        let reference = render(&single, &frame, &edits, &opts);
        let got = render(&budgeted, &frame, &edits, &opts);
        if budgeted.display_tiles() == before || budgeted.sensor_tiles() != 0 {
            panic!("a {label} 6000x4000 render in 320 MiB did not use display tiles alone");
        }
        require_identical(label, &reference, &got);
    }
}
