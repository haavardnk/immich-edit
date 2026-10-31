use raw_pipeline::edits::{
    BokehShape, CropRect, Edits, LensEdits, MaskComponentKind, MaskLayer, MaskedEdits,
};
use raw_pipeline::frame::{BitDepth, OutputFormat, PngCompression, RawFrame, RenderOptions};
use raw_pipeline::source::{self, SourceCoding};
use raw_pipeline::{GpuRenderer, TilePlan, timing};
use raw_pipeline_testkit::depth::{bokeh_frame, depth_rasters, lens_blur_edits};
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::masks::layer;
use raw_pipeline_testkit::render::rgb8_opts;

use super::{CANON_5D, Mode, Pair, XT2, decoded, radial, strokes};

const TILES: TilePlan = TilePlan::DisplayTiled { edge: 1024 };
const FIXTURES: [&str; 2] = [XT2, CANON_5D];
const BUDGET: u64 = 320 << 20;
const ROI: CropRect = CropRect {
    x: 0.3,
    y: 0.25,
    w: 0.4,
    h: 0.45,
};

fn exposure(ev: f64) -> MaskedEdits {
    MaskedEdits {
        exposure_ev: Some(ev),
        ..Default::default()
    }
}

fn masks() -> Vec<MaskLayer> {
    let range = MaskComponentKind::LumaRange {
        min: 0.4,
        max: 1.0,
        softness: 0.05,
    };
    vec![
        layer("radial", radial((0.4, 0.55), (0.3, 0.25)), exposure(0.6)),
        layer("range", range, exposure(-0.8)),
    ]
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

fn retouched() -> Edits {
    let mut edits = local_edits();
    edits.lens = LensEdits::default();
    edits.retouch = strokes();
    edits
}

fn cropped(crop: CropRect, rotate_angle: f32) -> Edits {
    let mut edits = local_edits();
    edits.geometry.rotate_angle = rotate_angle;
    edits.geometry.crop = Some(crop);
    edits
}

fn warped_edits() -> Edits {
    let crop = CropRect {
        x: 0.1,
        y: 0.05,
        w: 0.8,
        h: 0.85,
    };
    cropped(crop, 2.5)
}

fn tight_crop_edits() -> Edits {
    let crop = CropRect {
        x: 0.2,
        y: 0.15,
        w: 0.6,
        h: 0.6,
    };
    cropped(crop, 0.0)
}

fn masked_presence(id: &str, edits: MaskedEdits) -> MaskLayer {
    layer(id, radial((0.6, 0.4), (0.25, 0.3)), edits)
}

fn context_edits() -> Edits {
    let mut edits = warped_edits();
    edits.basic.dehaze = 35.0;
    edits.basic.texture = 40.0;
    edits.basic.clarity = 30.0;
    edits.tone.shadows = 45.0;
    let detail = MaskedEdits {
        texture: Some(-50.0),
        clarity: Some(60.0),
        shadows: Some(-30.0),
        ..Default::default()
    };
    edits.masks.push(masked_presence("detail", detail));
    edits
}

fn base_level_edits() -> Edits {
    let mut edits = local_edits();
    edits.basic.dehaze = -25.0;
    edits.tone.shadows = 40.0;
    let texture = MaskedEdits {
        texture: Some(70.0),
        ..Default::default()
    };
    edits.masks.push(masked_presence("texture", texture));
    edits
}

fn masked_shadows_edits() -> Edits {
    let mut edits = local_edits();
    let shadows = MaskedEdits {
        shadows: Some(60.0),
        ..Default::default()
    };
    edits.masks.push(masked_presence("shadows", shadows));
    edits
}

fn budget_edits() -> Edits {
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    edits.effects.grain_amount = 20.0;
    edits.effects.vignette_amount = -30.0;
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 30.0;
    edits.tone.shadows = 40.0;
    edits
}

fn export(max_edge: u32) -> RenderOptions {
    RenderOptions {
        quality: true,
        ..rgb8_opts(max_edge)
    }
}

fn binned(opts: RenderOptions) -> RenderOptions {
    RenderOptions {
        histogram: true,
        scopes: true,
        ..opts
    }
}

fn roi(max_edge: u32, roi: CropRect) -> RenderOptions {
    RenderOptions {
        roi: Some(roi),
        ..rgb8_opts(max_edge)
    }
}

fn modes(frame: &RawFrame) -> [Mode; 2] {
    let half = frame.meta.width.max(frame.meta.height) as u32 / 2;
    let sixteen = RenderOptions {
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        ..export(half)
    };
    [
        ("preview", binned(rgb8_opts(1600))),
        ("16-bit half export", binned(sixteen)),
    ]
}

#[test]
fn display_tiles_match_the_single_pass_on_real_raws() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    for name in FIXTURES {
        let frame = decoded(name);
        pair.require_modes(name, &frame, &warped_edits(), &modes(&frame));
    }
    let frame = decoded(CANON_5D);
    let preview = binned(rgb8_opts(1600));
    pair.require("5D retouched preview", &frame, &retouched(), &preview);
    let frame = decoded(XT2);
    let opts = export(frame.meta.width as u32);
    pair.require("X-T2 context export", &frame, &context_edits(), &opts);
}

#[test]
fn display_tiles_match_magnified_and_roi_previews_with_a_range_mask() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let cases = [
        (
            XT2,
            "magnified preview",
            tight_crop_edits(),
            rgb8_opts(1600),
        ),
        (CANON_5D, "roi preview", local_edits(), roi(1600, ROI)),
        (
            CANON_5D,
            "context roi preview",
            context_edits(),
            roi(1600, ROI),
        ),
    ];
    for (name, mode, edits, opts) in &cases {
        let frame = decoded(name);
        pair.require(&format!("{name} {mode}"), &frame, edits, opts);
    }
}

#[test]
fn display_tiles_with_a_shared_context_match_the_single_pass() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let frame = decoded(XT2);
    pair.require_modes(XT2, &frame, &context_edits(), &modes(&frame));
    let frame = decoded(CANON_5D);
    let cases = [
        ("context", context_edits()),
        ("base levels", base_level_edits()),
        ("masked shadows", masked_shadows_edits()),
    ];
    for (label, edits) in cases {
        pair.require(&format!("5D {label}"), &frame, &edits, &rgb8_opts(1600));
    }
}

#[test]
fn display_tiles_with_lens_blur_and_a_context_match_the_single_pass() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let opts = RenderOptions {
        rasters: depth_rasters(),
        ..export(1800)
    };
    let mut edits = lens_blur_edits(80.0, BokehShape::Hexagon, 40.0);
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 40.0;
    edits.tone.shadows = 35.0;
    edits.retouch = strokes();
    for orientation in [(false, false, false), (true, true, false)] {
        let mut frame = bokeh_frame(1800, 1200);
        frame.meta.orientation = orientation;
        let got = pair.require(&format!("lens blur {orientation:?}"), &frame, &edits, &opts);
        if !got.timings.iter().any(|t| t.stage == timing::TILE_CONTEXT) {
            panic!("the lens blur render {orientation:?} timed no tile context");
        }
    }
}

#[test]
fn display_tiles_render_a_mosaic_over_the_sensor_tile_budget() {
    let Some(pair) = Pair::budgeted(BUDGET) else {
        return;
    };
    let frame = synthetic_bayer_frame(6000, 4000, "RGGB");
    pair.require(
        "6000x4000 in 320 MiB",
        &frame,
        &budget_edits(),
        &export(6000),
    );
}

fn tile_source(
    renderer: &GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> Vec<u8> {
    match renderer.render_source(frame, edits, opts, None) {
        Ok(rendered) => {
            if !matches!(rendered.tiling.plan, TilePlan::DisplayTiled { .. }) {
                panic!("the tile source rendered as {:?}", rendered.tiling);
            }
            source::encode(&rendered.image, SourceCoding::Framed).unwrap()
        }
        Err(e) => panic!("tile source failed: {e}"),
    }
}

fn require_tile_source(
    pair: &Pair,
    label: &str,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) {
    let reference = match pair.single.render_source(frame, edits, opts, None) {
        Ok(rendered) => source::encode(&rendered.image, SourceCoding::Framed).unwrap(),
        Err(e) => panic!("{label}: single pass source failed: {e}"),
    };
    if tile_source(&pair.tiled, frame, edits, opts) != reference {
        panic!("{label}: the display-tiled tile source differs from the single pass");
    }
}

#[test]
fn display_tiles_render_tile_sources_like_the_single_pass() {
    let Some(pair) = Pair::forced(TILES) else {
        return;
    };
    let cases = [
        (XT2, "context", context_edits()),
        (CANON_5D, "retouched", retouched()),
    ];
    for (name, label, edits) in cases {
        let frame = decoded(name);
        let label = format!("{name} {label}");
        require_tile_source(&pair, &label, &frame, &edits, &roi(1600, ROI));
    }
}

#[test]
fn display_tiles_render_only_the_window_of_a_tile_source_over_the_budget() {
    let Some(pair) = Pair::budgeted(BUDGET) else {
        return;
    };
    let frame = synthetic_bayer_frame(6000, 4000, "RGGB");
    let viewport = CropRect {
        x: 0.55,
        y: 0.4,
        w: 0.2,
        h: 0.25,
    };
    let opts = roi(6000, viewport);
    require_tile_source(
        &pair,
        "tile source in 320 MiB",
        &frame,
        &budget_edits(),
        &opts,
    );
}
