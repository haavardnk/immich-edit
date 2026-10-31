use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;

use wgpu::TextureFormat;

use super::display::{OutputShape, final_pass_active, layer_presence};
use super::{GpuRenderer, RenderPlan, SensorShape, compose_edits, display_depth};
use crate::cpu::demosaic::parse_xtrans;
use crate::cpu::masked::{SelectorTaps, build_layer_eval};
use crate::edits::{Edits, MaskLayer, RetouchMode};
use crate::frame::{FrameMeta, PreviewMode, RawFrame, RenderOptions};
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::helpers::{mip_count, round_up_256};
use crate::gpu::readback::readback_size;
use crate::gpu::resources::{HISTOGRAM_BYTES, SCOPE_BYTES};
use crate::gpu::source::{RenderSource, SourceExtent};
use crate::gpu::texture::texture_size;
use crate::ops::ResolvedDcp;
use crate::ops::dehaze::DehazeGrid;
use crate::ops::lens_blur::kernel::BlurScale;
use crate::ops::presence::has_shadows;
use crate::source::LinearKind;
use crate::tile::{Rect, SensorTile, TilePlan, tile_strokes};

mod display_tiles;
mod host;
#[cfg(all(test, target_vendor = "apple"))]
mod tests;

const SMALL_RESOURCES: u64 = 32 << 20;
const SMALL_HOST: u64 = 4 << 20;
const R8: u64 = 1;
const R32: u64 = 4;
const RGBA8: u64 = 4;
const RGBA16: u64 = 8;
const RGBA32: u64 = 16;
const MOSAIC_COPIES: u64 = 2;
const NR_LUMA_PLANES: u64 = 4;
const NR_CHROMA_PLANES: u64 = 5;
const CAPTURE_PLANES: u64 = 4;
const DEHAZE_MOMENTS: u64 = 2;
const SHARPEN_TEXTURES: usize = 4;
const MASK_TEXEL: u64 = 2 * RGBA16 + 2 * R32;
const MASK_PREVIEW_TEXEL: u64 = R32 + RGBA8;
const LENS_BLUR_GRIDS: u64 = 2;
const SELECTOR_SCRATCH_TEXEL: u64 = RGBA32 + 4 * RGBA16;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Levels {
    Local,
    Context,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkingBytes {
    pub device: u64,
    pub host: u64,
}

struct WorkingSet {
    linear: TextureFormat,
    bytes: u64,
    host: u64,
    upload: Rc<Cell<u64>>,
}

impl WorkingSet {
    fn total(&self) -> WorkingBytes {
        WorkingBytes {
            device: self.bytes,
            host: SMALL_HOST + self.host.max(self.upload.get()),
        }
    }

    fn host_upload(&self, bytes: u64) {
        self.upload.set(self.upload.get().max(bytes));
    }

    fn texels(&mut self, (w, h): (u32, u32), texel: u64) {
        self.bytes += u64::from(w) * u64::from(h) * texel;
    }

    fn linear(&mut self, (w, h): (u32, u32), mips: bool) {
        let levels = if mips { mip_count(w, h) } else { 1 };
        self.bytes += texture_size(self.linear, (w, h), levels);
    }

    fn sensor(
        &mut self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        presence: bool,
        tiles: TilePlan,
    ) -> SourceExtent {
        let edits = edits.clamped();
        let shape = SensorShape::new(frame, &edits, options, presence);
        let working = shape.working();
        match tiles {
            TilePlan::SinglePass | TilePlan::DisplayTiled { .. } => {
                let tile = shape.chain.whole();
                self.sensor_tile(frame, &edits, options, &shape, &tile, presence);
            }
            TilePlan::SensorTiled { edge } => {
                self.linear(working, true);
                self.bytes += shape
                    .chain
                    .tiles(edge)
                    .iter()
                    .map(|tile| {
                        let mut set = self.empty();
                        set.sensor_tile(frame, &edits, options, &shape, tile, presence);
                        set.bytes
                    })
                    .max()
                    .unwrap_or(0);
            }
        }
        SourceExtent {
            dims: working,
            full: working,
        }
    }

    fn sensor_tile(
        &mut self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        shape: &SensorShape,
        tile: &SensorTile,
        presence: bool,
    ) {
        let input = tile.input().dims();
        self.upload(frame, shape, tile.input());
        if let Ok(Some(_)) = crate::ai_denoise::active_plane(frame, edits, options) {
            self.host_upload(host::denoise_plane(input));
            self.texels(input, RGBA16);
            self.linear(input, true);
        }
        for rect in &tile.rects[1..] {
            self.linear(rect.dims(), true);
        }
        if !presence {
            return;
        }
        let detail = tile.detail.dims();
        self.linear(tile.window().dims(), true);
        self.retouch(frame, edits, shape, tile);
        let luma_planes = self.noise_reduction(edits, detail);
        if shape.sigma.is_some() {
            self.texels(detail, CAPTURE_PLANES.saturating_sub(luma_planes) * R32);
            self.linear(detail, true);
        }
        if shape.resamples() {
            self.linear((tile.core.w, detail.1), false);
            self.linear(tile.core.dims(), true);
        }
    }

    fn upload(&mut self, frame: &RawFrame, shape: &SensorShape, input: Rect) {
        let window = shape.mosaic_window(frame, input);
        self.host_upload(host::upload(frame, window, shape.block.is_none()));
        let single_plane = shape.block.is_some() || frame.cpp != 3;
        if single_plane {
            let samples = u64::from(window.w) * u64::from(window.h) * frame.cpp as u64;
            self.bytes += MOSAIC_COPIES * samples * R32;
        } else {
            self.texels(input.dims(), RGBA16);
        }
        if shape.block.is_none() && parse_xtrans(&frame.cfa_pattern).is_some() {
            self.texels(input.dims(), R32);
        }
        self.linear(input.dims(), true);
    }

    fn retouch(&mut self, frame: &RawFrame, edits: &Edits, shape: &SensorShape, tile: &SensorTile) {
        let window = tile.window();
        let strokes = tile_strokes(edits, tile, shape.sensor(), frame.meta.orientation);
        for (i, (stroke, geom)) in strokes.iter().enumerate() {
            let patch = (
                (geom.bbox.x1 - geom.bbox.x0) as u32,
                (geom.bbox.y1 - geom.bbox.y0) as u32,
            );
            let heal = matches!(stroke.mode, RetouchMode::Heal);
            self.linear(patch, false);
            self.linear(patch, heal);
            self.linear(patch, heal);
            self.linear(window.dims(), i + 1 == strokes.len() && !tile.grown());
        }
        if tile.grown() {
            self.linear(tile.detail.dims(), true);
        }
    }

    fn noise_reduction(&mut self, edits: &Edits, dims: (u32, u32)) -> u64 {
        let luma = edits.detail.luma_nr_active();
        let chroma = edits.detail.color_nr_active();
        let luma_planes = if luma { NR_LUMA_PLANES } else { 0 };
        self.texels(dims, luma_planes * R32);
        if chroma {
            self.texels(
                (dims.0.div_ceil(2), dims.1.div_ceil(2)),
                NR_CHROMA_PLANES * RGBA32,
            );
        }
        if luma && chroma {
            self.linear(dims, false);
        }
        if luma || chroma {
            self.linear(dims, true);
        }
        luma_planes
    }

    fn empty(&self) -> Self {
        WorkingSet {
            linear: self.linear,
            bytes: 0,
            host: 0,
            upload: self.upload.clone(),
        }
    }

    fn display(
        &mut self,
        meta: &FrameMeta,
        source: (SourceExtent, LinearKind),
        edits: &Edits,
        opts: &RenderOptions,
        output: (OutputShape, Levels),
    ) {
        let edits = edits.clamped();
        let (extent, kind) = source;
        let (shape, levels) = output;
        let layers = if kind == LinearKind::PostWb {
            self.spatial(extent, &edits, opts, levels)
        } else {
            0
        };
        let dims = extent.dims;
        let full = (meta.width as u32, meta.height as u32);
        let out = shape.dims();
        if let Some(target) = shape.target(extent.full) {
            let work = Rect::full(dims).rescaled(extent.full, target).dims();
            for _ in 0..=layers {
                self.linear((work.0, dims.1), false);
                self.linear(work, false);
            }
        }
        let mut edits = edits;
        edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(meta.is_raw));
        let dcp = crate::dcp::setup::resolve(meta, &edits, opts.dcp.as_deref()).resolved;
        let long_edge = full.0.max(full.1);
        self.output(out, long_edge, &edits, opts, dcp.as_deref());
    }

    fn spatial(
        &mut self,
        extent: SourceExtent,
        edits: &Edits,
        opts: &RenderOptions,
        levels: Levels,
    ) -> usize {
        let dims = extent.dims;
        let local = levels == Levels::Local;
        if edits.basic.dehaze != 0.0 {
            self.linear(dims, false);
        }
        if edits.basic.dehaze != 0.0 && local {
            let scale = DehazeGrid::for_dims(extent.full).scale;
            let low = ((dims.0 / scale).max(1), (dims.1 / scale).max(1));
            self.linear(low, false);
            self.texels(low, DEHAZE_MOMENTS * RGBA32);
        }
        self.lens_blur(extent, edits, opts);
        let layers = layer_presence(edits);
        let layer_passes: HashSet<[u32; 3]> = layers
            .iter()
            .filter(|(_, _, amounts)| amounts.adjusts())
            .map(|(_, _, amounts)| amounts.bits())
            .collect();
        let global = usize::from(edits.basic.texture != 0.0 || edits.basic.clarity != 0.0);
        let presence_passes = global + layer_passes.len();
        if presence_passes > 0 && local {
            self.linear(dims, true);
        }
        for _ in 0..presence_passes {
            self.linear(dims, false);
        }
        if has_shadows(edits) {
            self.linear(dims, local);
        }
        layers.len()
    }

    fn lens_blur(&mut self, extent: SourceExtent, edits: &Edits, opts: &RenderOptions) {
        let blur = &edits.lens_blur;
        if !blur.active() {
            return;
        }
        let Some(depth) = opts.rasters.get(&blur.depth_raster_id) else {
            return;
        };
        let dims = extent.dims;
        let ds = BlurScale::new(blur.amount as f32, extent.full).ds;
        self.linear(dims, false);
        self.texels(
            (dims.0.div_ceil(ds) + 1, dims.1.div_ceil(ds) + 1),
            LENS_BLUR_GRIDS * RGBA32,
        );
        self.texels((depth.width, depth.height), R8);
    }

    fn output(
        &mut self,
        out: (u32, u32),
        long_edge: u32,
        edits: &Edits,
        opts: &RenderOptions,
        dcp: Option<&ResolvedDcp>,
    ) {
        let rounded = (round_up_256(out.0), round_up_256(out.1));
        self.texels(rounded, RGBA8 + RGBA16);
        self.bytes += readback_size(rounded.0, rounded.1, 4) + 2 * (HISTOGRAM_BYTES + SCOPE_BYTES);
        let preview = match &opts.preview_mode {
            PreviewMode::MaskWeight { layer_id } => edits.masks.iter().find(|l| &l.id == layer_id),
            _ => None,
        };
        let layers: Vec<&MaskLayer> = match preview {
            Some(layer) => vec![layer],
            None => edits.masks.iter().filter(|l| l.is_effective()).collect(),
        };
        let has_masks = preview.is_none() && !layers.is_empty();
        if final_pass_active(edits, opts, has_masks, dcp.is_some()) {
            for _ in 0..SHARPEN_TEXTURES {
                self.linear(rounded, false);
            }
        }
        let depth = display_depth(opts);
        let display_texel = u64::from(depth.format().block_copy_size(None).unwrap_or(0));
        if depth == DisplayDepth::Sixteen {
            self.texels(out, display_texel);
            self.bytes += readback_size(out.0, out.1, 8);
        }
        if edits.color.lut_3d.is_active() {
            self.texels(out, display_texel);
        }
        if dcp.is_some_and(|dcp| dcp.look_table.is_some() || dcp.tone_curve.is_some()) {
            self.texels(out, display_texel);
        }
        if preview.is_some() {
            self.texels(out, MASK_PREVIEW_TEXEL);
        }
        if has_masks {
            self.texels(out, MASK_TEXEL);
        }
        let aspect = out.0 as f32 / out.1.max(1) as f32;
        if layers
            .iter()
            .any(|l| build_layer_eval(l, &opts.rasters, aspect).reads_display())
        {
            let margin = 2 * SelectorTaps::for_long_edge(long_edge as f32).reach();
            let expanded = (out.0 + 2 * margin, out.1 + 2 * margin);
            self.texels(expanded, SELECTOR_SCRATCH_TEXEL + display_texel);
            self.texels(out, RGBA32);
        }
    }
}

impl GpuRenderer {
    fn working_set(&self) -> WorkingSet {
        WorkingSet {
            linear: self.ctx.linear_format,
            bytes: SMALL_RESOURCES,
            host: 0,
            upload: Rc::default(),
        }
    }

    pub fn estimate_render(
        &self,
        source: RenderSource<'_>,
        edits: &Edits,
        options: &RenderOptions,
        tiles: TilePlan,
    ) -> WorkingBytes {
        let edits = compose_edits(edits, options);
        let mut set = self.working_set();
        let linear = match source {
            RenderSource::Raw(frame) => {
                let presence = RenderPlan::select(&edits, frame) == RenderPlan::Presence;
                let kind = if presence {
                    LinearKind::PostWb
                } else {
                    LinearKind::PreWb
                };
                if let TilePlan::DisplayTiled { edge, held } = tiles {
                    set.display_tiles(frame, &edits, options, kind, (edge, held));
                    return set.total();
                }
                (set.sensor(frame, &edits, options, presence, tiles), kind)
            }
            RenderSource::Linear(linear) => (linear.extent(), linear.kind),
        };
        let shape = OutputShape::new(source.meta(), &edits, options.max_edge, linear.0.full);
        set.host = host::display(shape.dims(), display_depth(options), options);
        set.display(
            source.meta(),
            linear,
            &edits,
            options,
            (shape, Levels::Local),
        );
        set.total()
    }

    pub fn estimate_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        tiles: TilePlan,
    ) -> WorkingBytes {
        let edits = compose_edits(edits, options);
        let mut set = self.working_set();
        if options.roi.is_some() {
            set.source_window(frame, &edits, options, tiles);
            return set.total();
        }
        let extent = set.sensor(frame, &edits, options, true, tiles);
        set.bytes += readback_size(extent.dims.0, extent.dims.1, RGBA16 as u32);
        set.host = host::source(extent.dims);
        set.total()
    }
}
