use crate::cpu::demosaic::parse_xtrans;
use crate::cpu::masked::{LayerEval, build_layer_evals};
use crate::cpu::renderer::sensor_cacheable;
use crate::edits::{Edits, RetouchMode};
use crate::finish::working_set::finish_bytes;
use crate::frame::{BitDepth, PreviewMode, RawFrame, RenderOptions};
use crate::ops::presence::has_shadows;
use crate::ops::retouch::stroke_geometry;

use super::Prepared;

const SMALL: u64 = 16 << 20;
const RGB: u64 = 12;
const SOURCE: u64 = 18;
const XTRANS: u64 = 4;
const WARP: u64 = 12;
const COLOR_NR: u64 = 14;
const LUMA_NR: u64 = 17;
const CAPTURE: u64 = 23;
const RETOUCH_PATCH: u64 = 12;
const HEAL_PATCH: u64 = 40;
const DEHAZE: u64 = 4;
const LENS_BLUR: u64 = 6;
const PRESENCE: u64 = 18;
const SHADOWS: u64 = 22;
const SELECTOR: u64 = 28;
const MASK_WEIGHT: u64 = 40;
const SHARPEN: u64 = 24;
const SHARPEN_MASK: u64 = 8;
const OUTPUT_8: u64 = 3;
const OUTPUT_16: u64 = 9;

type Dims = (usize, usize);

struct Stage {
    bytes: u64,
    sensor: Dims,
    scene: Dims,
}

pub(super) fn render(
    frame: &RawFrame,
    prep: &Prepared,
    options: &RenderOptions,
    cached: Option<Dims>,
) -> u64 {
    let mask_weight = matches!(options.preview_mode, PreviewMode::MaskWeight { .. });
    let layers = if mask_weight {
        Vec::new()
    } else {
        build_layer_evals(&prep.edits.masks, &options.rasters, 1.0)
    };
    let held = RGB * (1 + layers.len() as u64);
    let stage = sensor_stage(frame, prep, held, cached);
    let pinned = if sensor_cacheable(options) { RGB } else { 0 };
    let spike = largest(&[
        (true, RGB),
        (mask_weight, MASK_WEIGHT),
        (layers.iter().any(LayerEval::reads_display), SELECTOR),
    ])
    .max(effects_spike(&prep.edits, options));
    let scene = if mask_weight {
        stage.sensor
    } else {
        stage.scene
    };
    let scene = bytes(scene, held + pinned + spike);
    with_margin(stage.bytes.max(scene).max(output(prep, options)))
}

pub(super) fn source(
    frame: &RawFrame,
    prep: &Prepared,
    options: &RenderOptions,
    cached: Option<Dims>,
) -> u64 {
    let layers = build_layer_evals(&prep.edits.masks, &options.rasters, 1.0);
    let stage = sensor_stage(frame, prep, RGB * (1 + layers.len() as u64), cached);
    with_margin(stage.bytes.max(bytes(stage.scene, RGB + SOURCE)))
}

fn bytes((w, h): Dims, per_px: u64) -> u64 {
    (w * h) as u64 * per_px
}

fn with_margin(bytes: u64) -> u64 {
    SMALL + bytes + bytes / 4
}

fn largest(stages: &[(bool, u64)]) -> u64 {
    stages
        .iter()
        .filter(|(active, _)| *active)
        .map(|&(_, per_px)| per_px)
        .max()
        .unwrap_or(0)
}

fn sensor_stage(frame: &RawFrame, prep: &Prepared, held: u64, cached: Option<Dims>) -> Stage {
    if let Some(dims) = cached {
        return Stage {
            bytes: 0,
            sensor: dims,
            scene: dims,
        };
    }
    let block = prep.block.unwrap_or(1);
    let grid = (frame.meta.width / block, frame.meta.height / block);
    let sensor = if frame.meta.orientation.0 {
        (grid.1, grid.0)
    } else {
        grid
    };
    let scene = prep
        .preview_dims(sensor)
        .map_or(sensor, |(w, h)| (w as usize, h as usize));
    let spike = largest(&[(scene != sensor, RGB)]).max(sensor_spike(frame, prep));
    let retouch = bytes(sensor, held) + retouch(&prep.edits, sensor);
    Stage {
        bytes: bytes(sensor, held + spike).max(retouch),
        sensor,
        scene,
    }
}

fn retouch(edits: &Edits, (w, h): Dims) -> u64 {
    let strokes = edits.retouch.iter().filter(|stroke| stroke.is_effective());
    strokes
        .filter_map(|stroke| {
            let bbox = stroke_geometry(stroke, w, h, (false, false, false))?.bbox;
            let per_px = match stroke.mode {
                RetouchMode::Heal => HEAL_PATCH,
                _ => RETOUCH_PATCH,
            };
            Some(bytes((bbox.x1 - bbox.x0, bbox.y1 - bbox.y0), per_px))
        })
        .max()
        .unwrap_or(0)
}

fn sensor_spike(frame: &RawFrame, prep: &Prepared) -> u64 {
    let edits = &prep.edits;
    let detail = &edits.detail;
    let xtrans =
        prep.block.is_none() && frame.cpp == 1 && parse_xtrans(&frame.cfa_pattern).is_some();
    let warps = frame.embedded_lens.warp().is_some()
        || edits.lens.distortion_active()
        || edits.lens.ca_active()
        || frame.meta.orientation != (false, false, false);
    let capture = detail.capture_sharpen && prep.ctx.render.capture_sigma.is_some();
    largest(&[
        (xtrans, XTRANS),
        (warps, WARP),
        (detail.color_nr_active(), COLOR_NR),
        (detail.luma_nr_active(), LUMA_NR),
        (capture, CAPTURE),
    ])
}

fn effects_spike(edits: &Edits, options: &RenderOptions) -> u64 {
    let presence = edits.basic.texture != 0.0
        || edits.basic.clarity != 0.0
        || edits
            .masks
            .iter()
            .filter(|l| l.is_effective())
            .any(|l| l.edits.texture.is_some() || l.edits.clarity.is_some());
    let blur = &edits.lens_blur;
    let blurs = blur.active() && options.rasters.contains_key(&blur.depth_raster_id);
    largest(&[
        (edits.basic.dehaze != 0.0, DEHAZE),
        (presence, PRESENCE),
        (has_shadows(edits), SHADOWS),
        (blurs, LENS_BLUR),
    ])
}

fn output(prep: &Prepared, options: &RenderOptions) -> u64 {
    let out = (prep.out_dims.0 as usize, prep.out_dims.1 as usize);
    let edits = &prep.edits;
    let detail = &edits.detail;
    let masking = if detail.sharpen_masking > 0.0 {
        SHARPEN_MASK
    } else {
        0
    };
    let sharpens = detail.sharpen_active() || edits.masked_sharpen_active();
    let sharpen = largest(&[(sharpens, SHARPEN + masking)]);
    let sixteen = options.output.bit_depth() == BitDepth::Sixteen;
    let texel = if sixteen { OUTPUT_16 } else { OUTPUT_8 };
    let finish = bytes(out, texel) + finish_bytes(sixteen, prep.out_dims, options);
    bytes(out, RGB + sharpen).max(finish)
}
