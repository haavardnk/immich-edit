use std::iter::once;

use crate::encode::{EncodeInput, RGB8, RGB16, RGBA8, encode_bytes, pixel_bytes};
use crate::finish::has_final_stage;
use crate::finish::working_set::finish_bytes;
use crate::frame::{RawFrame, RenderOptions};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::renderer::display_tiles::ContextPlan;
use crate::tile::Rect;

const F32: u64 = 4;
const RGBA_F16: u64 = 8;
const DENOISE_PLANE: u64 = 14;
const SOURCE: u64 = 6;
const CONTEXT: u64 = 48;

pub(super) fn upload(frame: &RawFrame, window: Rect, rgba: bool) -> u64 {
    let whole = Rect::full((frame.meta.width as u32, frame.meta.height as u32));
    let copy = if window == whole {
        0
    } else {
        pixel_bytes(window.dims(), F32 * frame.cpp as u64)
    };
    let rgba = if rgba && frame.cpp == 3 {
        pixel_bytes(window.dims(), RGBA_F16)
    } else {
        0
    };
    copy + rgba
}

pub(super) fn denoise_plane(input: (u32, u32)) -> u64 {
    pixel_bytes(input, DENOISE_PLANE)
}

pub(super) fn held(out: (u32, u32), depth: DisplayDepth) -> u64 {
    let texel = match depth {
        DisplayDepth::Sixteen => RGB16,
        DisplayDepth::Eight => RGBA8,
    };
    pixel_bytes(out, texel)
}

pub(super) fn display(out: (u32, u32), depth: DisplayDepth, options: &RenderOptions) -> u64 {
    let sixteen = depth == DisplayDepth::Sixteen;
    let finished = match (has_final_stage(out.0, out.1, options), sixteen) {
        (true, true) => finish_bytes(true, out, options),
        (true, false) => pixel_bytes(out, RGB8) + finish_bytes(false, out, options),
        (false, true) => encode_bytes(EncodeInput::Rgb16, &options.output, out),
        (false, false) => encode_bytes(EncodeInput::Rgba8, &options.output, out),
    };
    held(out, depth) + finished
}

pub(super) fn source(dims: (u32, u32)) -> u64 {
    pixel_bytes(dims, SOURCE)
}

pub(super) fn context(plan: &ContextPlan) -> u64 {
    let levels = plan
        .coarse()
        .map(|coarse| plan.level_dims(plan.level(coarse)));
    once(plan.lo())
        .chain(levels)
        .map(|dims| pixel_bytes(dims, CONTEXT))
        .sum()
}
