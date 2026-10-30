use raw_pipeline::CpuRenderer;
use raw_pipeline::GpuRenderer;
use raw_pipeline::edits::{
    BokehShape, CropRect, DetailEdits, Edits, GeometryEdits, MaskComponent, MaskComponentKind,
    MaskComponentMode, MaskLayer, MaskSource, MaskedEdits, Vec2f, WhiteBalance,
};
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::source::{self, LinearKind, SourceImage};
use raw_pipeline_testkit::depth::{depth_rasters, lens_blur_edits};
use raw_pipeline_testkit::fixtures::first_fixture_frame;
use raw_pipeline_testkit::frames::{haze_frame, split_tone_frame};
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::parity::mean_abs_delta;
use raw_pipeline_testkit::render::rgb8_opts;

fn layer() -> MaskLayer {
    MaskLayer {
        id: "warm".into(),
        name: String::new(),
        enabled: true,
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "c1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Linear {
                p0: Vec2f { x: 0.0, y: 0.5 },
                p1: Vec2f { x: 1.0, y: 0.5 },
                feather: 0.4,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            wb_temp: Some(20.0),
            texture: Some(30.0),
            exposure_ev: Some(0.3),
            ..Default::default()
        },
    }
}

fn display_edits() -> Edits {
    let mut edits = Edits::default();
    edits.basic.exposure_ev = 0.4;
    edits.basic.contrast = 20.0;
    edits.basic.white_balance = WhiteBalance::Relative {
        temp: 15.0,
        tint: -10.0,
    };
    edits.basic.dehaze = 25.0;
    edits.basic.texture = 30.0;
    edits.basic.clarity = -20.0;
    edits.basic.vibrance = 15.0;
    edits.tone.shadows = 40.0;
    edits.tone.highlights = -30.0;
    edits.color.hsl.bands[2].sat = 25.0;
    edits.color.color_grade.shadows.sat = 20.0;
    edits.detail.sharpen_amount = Some(60.0);
    edits.effects.vignette_amount = -30.0;
    edits.effects.grain_amount = 20.0;
    edits.geometry.flip_h = true;
    edits.masks = vec![layer()];
    edits
}

fn all_edits() -> Edits {
    let display = display_edits();
    Edits {
        lens_blur: lens_blur_edits(60.0, BokehShape::Octagon, 20.0).lens_blur,
        detail: DetailEdits {
            luma_nr_amount: 35.0,
            color_nr_amount: 25.0,
            ..display.detail
        },
        geometry: GeometryEdits {
            crop: Some(CropRect {
                x: 0.1,
                y: 0.05,
                w: 0.8,
                h: 0.85,
            }),
            ..display.geometry.clone()
        },
        ..display
    }
}

fn frames() -> Vec<(&'static str, RawFrame)> {
    let mut rotated = haze_frame(112, 160);
    rotated.meta.orientation = (true, false, true);
    let mut out = vec![("haze", haze_frame(160, 112)), ("rotated", rotated)];
    if let Some(frame) = first_fixture_frame() {
        out.push(("fixture", frame));
    }
    out
}

fn wire(image: &SourceImage) -> Vec<u8> {
    source::encode(image, source::SourceCoding::Framed).unwrap()
}

fn gpu_source(
    renderer: &GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> SourceImage {
    renderer
        .render_source(frame, &edits.sensor_stage(), opts, None)
        .unwrap()
        .image
}

#[test]
fn a_source_survives_the_wire_and_renders_like_its_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(256);
    for (label, frame) in frames() {
        for (case, edits) in [("display", display_edits()), ("all", all_edits())] {
            let bytes = wire(&gpu_source(&renderer, &frame, &edits, &opts));
            let decoded = source::decode(&bytes).unwrap();
            if decoded.header.kind != LinearKind::PostWb || decoded.header.atmosphere.is_none() {
                panic!("{label} {case}: a source must be white balanced and carry an atmosphere");
            }
            let uploaded = renderer.upload_source(&decoded).unwrap();
            if wire(&renderer.read_source(&uploaded, None, None).unwrap()) != bytes {
                panic!("{label} {case}: uploading and reading back changed the source");
            }
            let from_frame = renderer.render(&frame, &edits, &opts).unwrap();
            let from_wire = renderer.render(&uploaded, &edits, &opts).unwrap();
            if from_frame.bytes != from_wire.bytes {
                panic!(
                    "{label} {case}: the wire source moved the render by {:.4}",
                    mean_abs_delta(&from_frame.bytes, &from_wire.bytes)
                );
            }
        }
    }
}

#[test]
fn a_source_stays_close_to_fast_plan_renders() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(256);
    let mut exposure = Edits::default();
    exposure.basic.exposure_ev = 0.5;
    for (label, frame) in frames() {
        for edits in [Edits::default(), exposure.clone()] {
            let image = gpu_source(&renderer, &frame, &edits, &opts);
            let uploaded = renderer.upload_source(&image).unwrap();
            let from_frame = renderer.render(&frame, &edits, &opts).unwrap();
            let from_source = renderer.render(&uploaded, &edits, &opts).unwrap();
            let delta = mean_abs_delta(&from_frame.bytes, &from_source.bytes);
            if delta > 1.0 {
                panic!("{label}: the source drifted {delta:.3} from a fast plan render");
            }
        }
    }
}

#[test]
fn display_edits_do_not_reach_the_source() {
    let cpu = CpuRenderer::new();
    let gpu = try_renderer();
    let opts = rgb8_opts(256);
    for (label, frame) in frames() {
        let full = all_edits();
        let subset = full.sensor_stage();
        let cpu_full = wire(&cpu.render_source(&frame, &full, &opts, None).unwrap().image);
        let cpu_subset = wire(
            &cpu.render_source(&frame, &subset, &opts, None)
                .unwrap()
                .image,
        );
        if cpu_full != cpu_subset {
            panic!("{label}: a field outside Edits::sensor_stage changed the cpu source");
        }
        let Some(gpu) = gpu.as_ref() else {
            continue;
        };
        let gpu_full = wire(&gpu.render_source(&frame, &full, &opts, None).unwrap().image);
        let gpu_subset = wire(
            &gpu.render_source(&frame, &subset, &opts, None)
                .unwrap()
                .image,
        );
        if gpu_full != gpu_subset {
            panic!("{label}: a field outside Edits::sensor_stage changed the gpu source");
        }
    }
}

#[test]
fn a_cpu_source_renders_like_the_gpu_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let cpu = CpuRenderer::new();
    let opts = rgb8_opts(256);
    for (label, frame) in frames() {
        for (case, edits) in [("display", display_edits()), ("all", all_edits())] {
            let image = cpu
                .render_source(&frame, &edits.sensor_stage(), &opts, None)
                .unwrap()
                .image;
            let uploaded = renderer
                .upload_source(&source::decode(&wire(&image)).unwrap())
                .unwrap();
            let from_frame = renderer.render(&frame, &edits, &opts).unwrap();
            let from_cpu = renderer.render(&uploaded, &edits, &opts).unwrap();
            if (from_frame.width, from_frame.height) != (from_cpu.width, from_cpu.height) {
                panic!("{label} {case}: the cpu source rendered at different dims");
            }
            let delta = mean_abs_delta(&from_frame.bytes, &from_cpu.bytes);
            if delta > 2.0 {
                panic!("{label} {case}: the cpu source drifted {delta:.3} from the gpu render");
            }
        }
    }
}

fn speckled_haze(w: usize, h: usize) -> RawFrame {
    let mut frame = haze_frame(w, h);
    for (i, v) in frame.data.iter_mut().enumerate() {
        let hash = (i as u32).wrapping_mul(2_654_435_761).rotate_left(13) ^ 0x9e37_79b9;
        let noise = (hash.wrapping_mul(1_103_515_245) >> 8) as f32 / (1u32 << 24) as f32;
        *v = (*v * 0.6 * (0.6 + 0.8 * noise)).min(1.0);
    }
    frame
}

#[test]
fn a_preview_source_estimates_the_cpu_atmosphere() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let cpu = CpuRenderer::new();
    let frame = speckled_haze(1600, 1200);
    let mut edits = Edits::default();
    edits.basic.dehaze = 40.0;
    for max_edge in [800, 1600] {
        let opts = rgb8_opts(max_edge);
        let gpu = renderer.render_source(&frame, &edits, &opts, None).unwrap();
        let cpu = cpu.render_source(&frame, &edits, &opts, None).unwrap();
        let (Some(gpu_atm), Some(cpu_atm)) =
            (gpu.image.header.atmosphere, cpu.image.header.atmosphere)
        else {
            panic!("{max_edge}: a source must carry an atmosphere");
        };
        if gpu_atm
            .iter()
            .zip(cpu_atm)
            .any(|(g, c)| (g - c).abs() > 0.01)
        {
            panic!("{max_edge}: gpu atmosphere {gpu_atm:?} differs from cpu {cpu_atm:?}");
        }
    }
}

fn tile_opts(max_edge: u32, roi: CropRect) -> RenderOptions {
    RenderOptions {
        roi: Some(roi),
        rasters: depth_rasters(),
        ..rgb8_opts(max_edge)
    }
}

fn tile_cases() -> Vec<(&'static str, Edits)> {
    let mut tilted = all_edits();
    tilted.geometry.rotate_angle = 3.0;
    let blurred = Edits {
        lens_blur: lens_blur_edits(100.0, BokehShape::Hexagon, 60.0).lens_blur,
        ..display_edits()
    };
    let mut ranged = display_edits();
    ranged.masks[0].components[0].kind = MaskComponentKind::LumaRange {
        min: 0.3,
        max: 0.8,
        softness: 0.1,
    };
    vec![
        ("display", display_edits()),
        ("tilted", tilted),
        ("lens blur", blurred),
        ("range mask", ranged),
    ]
}

#[test]
fn a_windowed_source_renders_the_tile_of_the_full_source() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let cpu = CpuRenderer::new();
    let mut turned = haze_frame(800, 1200);
    turned.meta.orientation = (true, false, true);
    let rois = [
        tile_roi(),
        CropRect {
            x: 0.4,
            ..tile_roi()
        },
    ];
    let frames = [
        ("haze", haze_frame(1200, 800)),
        ("odd", haze_frame(1203, 805)),
        ("split", split_tone_frame(1203, 805)),
        ("turned", turned),
    ];
    for (label, frame) in frames {
        let long = frame.meta.width.max(frame.meta.height) as u32;
        let cases = tile_cases().into_iter().flat_map(|case| {
            rois.into_iter()
                .flat_map(move |roi| [4, 10].map(|shrink| (case.clone(), roi, shrink)))
        });
        for ((case, edits), roi, shrink) in cases {
            let at = roi.x;
            let sensor = edits.sensor_stage();
            let tile = tile_opts(long / shrink, roi);
            let sources = [
                (
                    "gpu",
                    renderer.render_source(&frame, &edits, &tile, None),
                    renderer.render_source(&frame, &sensor, &rgb8_opts(long), None),
                ),
                (
                    "cpu",
                    cpu.render_source(&frame, &edits, &tile, None),
                    cpu.render_source(&frame, &sensor, &rgb8_opts(long), None),
                ),
            ];
            let mut areas = Vec::new();
            for (kind, windowed, full) in sources {
                let windowed = source::decode(&wire(&windowed.unwrap().image)).unwrap();
                let full = source::decode(&wire(&full.unwrap().image)).unwrap();
                let Some(window) = windowed.header.window else {
                    panic!("{label} {case} {at} 1/{shrink} {kind}: a small tile was not windowed");
                };
                if window.full != full.header.dims || windowed.header.dims.0 >= window.full.0 {
                    panic!(
                        "{label} {case} {at} 1/{shrink} {kind}: window {window:?} does not sit in the full source"
                    );
                }
                let exact = kind == "gpu";
                if windowed.header.context.is_some() != exact {
                    panic!("{label} {case} {at} 1/{shrink} {kind}: only gpu tiles carry a context");
                }
                areas.push(windowed.header.dims.0 * windowed.header.dims.1);
                let from_window = renderer
                    .render(&renderer.upload_source(&windowed).unwrap(), &edits, &tile)
                    .unwrap();
                let from_full = renderer
                    .render(&renderer.upload_source(&full).unwrap(), &edits, &tile)
                    .unwrap();
                if (from_window.width, from_window.height) != (from_full.width, from_full.height) {
                    panic!(
                        "{label} {case} {at} 1/{shrink} {kind}: the tile rendered at different dims"
                    );
                }
                let delta = mean_abs_delta(&from_window.bytes, &from_full.bytes);
                let worst = from_window
                    .bytes
                    .iter()
                    .zip(&from_full.bytes)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap_or(0);
                if (exact && worst > 0) || delta > 0.01 || worst > 2 {
                    panic!(
                        "{label} {case} {at} 1/{shrink} {kind}: the windowed tile drifted {delta:.4}, worst {worst}"
                    );
                }
            }
            if areas[0] * 2 > areas[1] {
                panic!(
                    "{label} {case} {at} 1/{shrink}: a context window of {} px is not far below the spatial {}",
                    areas[0], areas[1]
                );
            }
        }
    }
}

fn tile_roi() -> CropRect {
    CropRect {
        x: 0.15,
        y: 0.35,
        w: 0.2,
        h: 0.25,
    }
}

#[test]
fn a_tile_source_follows_its_context_key() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = speckled_haze(1203, 805);
    let tile = tile_opts(1203, tile_roi());
    let base = display_edits();
    let mut outside = base.clone();
    outside.basic.contrast = -15.0;
    outside.basic.vibrance = 40.0;
    outside.effects.grain_amount = 0.0;
    outside.detail.sharpen_amount = Some(10.0);
    outside.masks[0].edits.exposure_ev = Some(-0.5);
    let mut inside = base.clone();
    inside.basic.dehaze = 50.0;
    let bytes = |edits: &Edits| {
        let rendered = renderer.render_source(&frame, edits, &tile, None).unwrap();
        wire(&rendered.image)
    };
    let key = source::context_key(&base);
    if source::context_key(&outside) != key || bytes(&outside) != bytes(&base) {
        panic!("an edit outside the context key changed the tile source");
    }
    if source::context_key(&inside) == key || bytes(&inside) == bytes(&base) {
        panic!("a dehaze change left the tile source or its key unchanged");
    }
}

#[test]
fn a_stale_tile_context_still_renders() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = split_tone_frame(1203, 805);
    let tile = tile_opts(1203, tile_roi());
    let mut haze = Edits::default();
    haze.basic.dehaze = 30.0;
    let mut moved = display_edits();
    moved.basic.dehaze = -40.0;
    moved.basic.clarity = 35.0;
    for (label, built, drawn) in [
        ("moved sliders", display_edits(), moved),
        ("new needs", haze, display_edits()),
    ] {
        let image = renderer
            .render_source(&frame, &built, &tile, None)
            .unwrap()
            .image;
        let uploaded = renderer
            .upload_source(&source::decode(&wire(&image)).unwrap())
            .unwrap();
        let fresh = renderer.render(&frame, &drawn, &tile).unwrap();
        match renderer.render(&uploaded, &drawn, &tile) {
            Ok(stale) if (stale.width, stale.height) == (fresh.width, fresh.height) => {}
            Ok(stale) => panic!(
                "{label}: a stale tile rendered {}x{}",
                stale.width, stale.height
            ),
            Err(e) => panic!("{label}: a stale tile failed to render: {e}"),
        }
    }
}

#[test]
fn a_pan_reuses_the_tile_context_until_its_key_changes() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = split_tone_frame(1203, 805);
    let builds = |edits: &Edits, x: f32| {
        let tile = tile_opts(1203, CropRect { x, ..tile_roi() });
        let rendered = renderer.render_source(&frame, edits, &tile, None).unwrap();
        rendered
            .timings
            .iter()
            .any(|t| t.stage == raw_pipeline::timing::TILE_CONTEXT)
    };
    let edits = display_edits();
    let mut moved = edits.clone();
    moved.basic.dehaze = 45.0;
    for (label, edits, x, expected) in [
        ("first tile", &edits, 0.4, true),
        ("pan", &edits, 0.1, false),
        ("dehaze change", &moved, 0.1, true),
    ] {
        if builds(edits, x) != expected {
            panic!("{label}: built a context is {}", !expected);
        }
    }
}

#[test]
fn a_dehaze_change_draws_exactly_from_the_tile_already_loaded() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = speckled_haze(1203, 805);
    let tile = tile_opts(1203, tile_roi());
    let mut dehazed = Edits::default();
    dehazed.basic.dehaze = 35.0;
    if source::context_key(&dehazed) != source::context_key(&Edits::default()) {
        panic!("a dehaze change without levels must keep the tile's key");
    }
    let uploaded = |edits: &Edits| {
        let image = renderer
            .render_source(&frame, edits, &tile, None)
            .unwrap()
            .image;
        let haze = image
            .header
            .context
            .and_then(|c| c.get(source::ContextPart::Haze));
        if haze.is_none() {
            panic!("a tile source must always carry the haze grid");
        }
        renderer
            .upload_source(&source::decode(&wire(&image)).unwrap())
            .unwrap()
    };
    let loaded = renderer
        .render(&uploaded(&Edits::default()), &dehazed, &tile)
        .unwrap();
    let fresh = renderer
        .render(&uploaded(&dehazed), &dehazed, &tile)
        .unwrap();
    if loaded.bytes != fresh.bytes {
        panic!(
            "the loaded tile drew dehaze {:.4} away from a fresh one",
            mean_abs_delta(&loaded.bytes, &fresh.bytes)
        );
    }
}

#[test]
fn a_full_size_rotated_range_mask_tile_renders_like_the_server_tile() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = speckled_haze(6000, 4000);
    let tile = tile_opts(6000, tile_roi());
    let (_, mut ranged) = tile_cases().pop().unwrap();
    ranged.geometry.rotate_angle = 3.0;
    let image = renderer
        .render_source(&frame, &ranged, &tile, None)
        .unwrap()
        .image;
    let uploaded = renderer
        .upload_source(&source::decode(&wire(&image)).unwrap())
        .unwrap();
    let from_tile = renderer.render(&uploaded, &ranged, &tile).unwrap();
    let from_frame = renderer.render(&frame, &ranged, &tile).unwrap();
    if from_tile.bytes != from_frame.bytes {
        panic!(
            "the range mask tile drifted {:.4} from the server tile",
            mean_abs_delta(&from_tile.bytes, &from_frame.bytes)
        );
    }
}
