use std::collections::HashSet;

use wgpu::TextureFormat;

use super::display::{final_pass_active, layer_presence, scaled_dims};
use super::upload::sensor_dims;
use super::{GpuRenderer, RenderPlan, compose_edits, display_depth};
use crate::cpu::demosaic::parse_xtrans;
use crate::cpu::masked::{SelectorTaps, build_layer_eval};
use crate::edits::{Edits, MaskLayer, RetouchMode};
use crate::frame::{FrameMeta, PreviewMode, RawFrame, RenderOptions};
use crate::geom;
use crate::gpu::display_depth::DisplayDepth;
use crate::gpu::helpers::{mip_count, round_up_256};
use crate::gpu::readback::readback_size;
use crate::gpu::resources::{HISTOGRAM_BYTES, SCOPE_BYTES};
use crate::gpu::source::{RenderSource, SourceExtent};
use crate::gpu::texture::texture_size;
use crate::ops::ResolvedDcp;
use crate::ops::capture_sharpen::frame_sigma;
use crate::ops::dehaze::DehazeGrid;
use crate::ops::lens_blur::kernel::BlurScale;
use crate::ops::presence::has_shadows;
use crate::ops::retouch::stroke_geometry;
use crate::source::LinearKind;

#[cfg(all(test, target_vendor = "apple"))]
mod tests;

const SMALL_RESOURCES: u64 = 32 << 20;
const R8: u64 = 1;
const R32: u64 = 4;
const RGBA8: u64 = 4;
const RGBA16: u64 = 8;
const RGBA32: u64 = 16;
const MOSAIC_COPIES: u64 = 2;
const NR_LUMA_PLANES: u64 = 4;
const NR_CHROMA_PLANES: u64 = 5;
const CAPTURE_PLANES: u64 = 4;
const DEHAZE_MOMENTS: u64 = 9;
const SHARPEN_TEXTURES: usize = 4;
const MASK_TEXEL: u64 = 2 * RGBA16 + 2 * R32;
const MASK_PREVIEW_TEXEL: u64 = R32 + RGBA8;
const LENS_BLUR_GRIDS: u64 = 2;
const SELECTOR_SCRATCH_TEXEL: u64 = RGBA32 + 4 * RGBA16;

struct WorkingSet {
    linear: TextureFormat,
    bytes: u64,
}

impl WorkingSet {
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
    ) -> SourceExtent {
        let full = (frame.meta.width as u32, frame.meta.height as u32);
        let ratio = geom::preview_ratio(
            frame.meta.orientation,
            edits,
            full,
            options.max_edge,
            options.quality,
        );
        let block = geom::superpixel_block(frame, options, ratio).unwrap_or(1);
        let dims = sensor_dims(frame, block);
        self.upload(frame, block, dims);
        let edits = edits.clamped();
        let ai_denoise = crate::ai_denoise::active_plane(frame, &edits, options);
        if let Ok(Some((plane, _))) = ai_denoise {
            let (w, h) = plane.binned_dims(block);
            self.texels((w as u32, h as u32), RGBA16);
            self.linear(dims, true);
        }
        if frame.embedded_lens.warp().is_some() {
            self.linear(dims, true);
        }
        if edits.lens.any_active() {
            self.linear(dims, true);
        }
        if !presence {
            return SourceExtent { dims, full: dims };
        }
        self.linear(dims, true);
        self.retouch(frame, &edits, dims);
        let luma_planes = self.noise_reduction(&edits, dims);
        if frame_sigma(frame, &edits, block).is_some() {
            self.texels(dims, CAPTURE_PLANES.saturating_sub(luma_planes) * R32);
            self.linear(dims, true);
        }
        let preview = ratio.and_then(|ratio| geom::resample_target(dims, ratio / block as f32));
        let Some(preview) = preview else {
            return SourceExtent { dims, full: dims };
        };
        self.linear((preview.0, dims.1), false);
        self.linear(preview, true);
        SourceExtent {
            dims: preview,
            full: preview,
        }
    }

    fn upload(&mut self, frame: &RawFrame, block: usize, dims: (u32, u32)) {
        if block == 1 && frame.cpp == 3 {
            self.texels(dims, RGBA16);
        } else {
            self.bytes += MOSAIC_COPIES * frame.data.len() as u64 * 4;
        }
        if block == 1 && parse_xtrans(&frame.cfa_pattern).is_some() {
            self.texels(dims, R32);
        }
        self.linear(dims, true);
    }

    fn retouch(&mut self, frame: &RawFrame, edits: &Edits, dims: (u32, u32)) {
        let (w, h) = (dims.0 as usize, dims.1 as usize);
        for stroke in edits.retouch.iter().filter(|s| s.is_effective()) {
            let Some(geom) = stroke_geometry(stroke, w, h, frame.meta.orientation) else {
                continue;
            };
            let patch = (
                (geom.bbox.x1 - geom.bbox.x0) as u32,
                (geom.bbox.y1 - geom.bbox.y0) as u32,
            );
            let heal = matches!(stroke.mode, RetouchMode::Heal);
            self.linear(patch, false);
            self.linear(patch, heal);
            self.linear(patch, heal);
            self.linear(dims, true);
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

    fn display(
        &mut self,
        meta: &FrameMeta,
        source: (SourceExtent, LinearKind),
        edits: &Edits,
        opts: &RenderOptions,
    ) {
        let edits = edits.clamped();
        let (extent, kind) = source;
        let layers = if kind == LinearKind::PostWb {
            self.spatial(extent, &edits, opts)
        } else {
            0
        };
        let dims = extent.dims;
        let full = (meta.width as u32, meta.height as u32);
        let out = geom::display_out_dims(meta.orientation, &edits, full, opts.max_edge);
        let (crop_w, crop_h) = geom::display_crop_px(meta.orientation, &edits, extent.full);
        let ratio = (crop_w as f32 / out.0 as f32).max(crop_h as f32 / out.1 as f32);
        if let Some(target) = geom::resample_target(extent.full, ratio) {
            let work = scaled_dims(dims, extent.full, target);
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

    fn spatial(&mut self, extent: SourceExtent, edits: &Edits, opts: &RenderOptions) -> usize {
        let dims = extent.dims;
        if edits.basic.dehaze != 0.0 {
            let scale = DehazeGrid::for_dims(extent.full).scale;
            let low = ((dims.0 / scale).max(1), (dims.1 / scale).max(1));
            self.linear(dims, false);
            self.linear(low, false);
            self.texels(low, DEHAZE_MOMENTS * RGBA32);
        }
        let blur = &edits.lens_blur;
        if blur.active()
            && let Some(depth) = opts.rasters.get(&blur.depth_raster_id)
        {
            let ds = BlurScale::new(blur.amount as f32, extent.full).ds;
            self.linear(dims, false);
            self.texels(
                (dims.0.div_ceil(ds) + 1, dims.1.div_ceil(ds) + 1),
                LENS_BLUR_GRIDS * RGBA32,
            );
            self.texels((depth.width, depth.height), R8);
        }
        let layers = layer_presence(edits);
        let layer_passes: HashSet<[u32; 3]> = layers
            .iter()
            .filter(|(_, _, amounts)| amounts.adjusts())
            .map(|(_, _, amounts)| amounts.bits())
            .collect();
        let global = usize::from(edits.basic.texture != 0.0 || edits.basic.clarity != 0.0);
        let presence_passes = global + layer_passes.len();
        if presence_passes > 0 {
            self.linear(dims, true);
        }
        for _ in 0..presence_passes {
            self.linear(dims, false);
        }
        if has_shadows(edits) {
            self.linear(dims, true);
        }
        layers.len()
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
        }
    }

    pub(super) fn estimate_render(
        &self,
        source: RenderSource<'_>,
        edits: &Edits,
        options: &RenderOptions,
    ) -> u64 {
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
                (set.sensor(frame, &edits, options, presence), kind)
            }
            RenderSource::Linear(linear) => (linear.extent(), linear.kind),
        };
        set.display(source.meta(), linear, &edits, options);
        set.bytes
    }

    pub(super) fn estimate_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
    ) -> u64 {
        let edits = compose_edits(edits, options);
        let mut set = self.working_set();
        let extent = set.sensor(frame, &edits, options, true);
        set.bytes += readback_size(extent.dims.0, extent.dims.1, RGBA16 as u32);
        set.bytes
    }
}
