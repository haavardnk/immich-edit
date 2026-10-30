use raw_pipeline::TilePlan;
use raw_pipeline::edits::{CropRect, Edits, MaskComponentKind, MaskLayer, MaskedEdits};
use raw_pipeline::frame::{BitDepth, OutputFormat, PngCompression, RawFrame, RenderOptions};
use raw_pipeline_testkit::frames::synthetic_bayer_frame;
use raw_pipeline_testkit::masks::layer;
use raw_pipeline_testkit::render::rgb8_opts;

use super::{CANON_5D, Mode, Pair, XT2, decoded, radial};

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

fn budget_edits() -> Edits {
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    edits.effects.grain_amount = 20.0;
    edits.effects.vignette_amount = -30.0;
    edits
}

fn export(max_edge: u32) -> RenderOptions {
    RenderOptions {
        quality: true,
        ..rgb8_opts(max_edge)
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
        ("preview", rgb8_opts(1600)),
        ("16-bit half export", sixteen),
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
    let preview = rgb8_opts(1600);
    pair.require("5D unwarped preview", &frame, &local_edits(), &preview);
    let frame = decoded(XT2);
    let opts = export(frame.meta.width as u32);
    pair.require("X-T2 export", &frame, &warped_edits(), &opts);
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
    ];
    for (name, mode, edits, opts) in &cases {
        let frame = decoded(name);
        pair.require(&format!("{name} {mode}"), &frame, edits, opts);
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
