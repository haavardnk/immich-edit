use crate::cancel::{self, CancelToken};
use crate::cpu::fused::{CpuFusedOp, FusedSegment, apply_segment};
use crate::cpu::masked::{blend_layer_images, build_layer_evals, effective_edits_for_layer};
use crate::cpu::presence::has_presence;
use crate::cpu::presence_pyramid::LumaPyramid;
use crate::cpu::transform;
use crate::edits::Edits;
use crate::ops::LinearImage;
use crate::ops::lens_distortion::LensWarpParams;
use crate::ops::{GpuRoute, OpContext, OpScratch, RenderContext, default_registry};
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};
use std::sync::Arc;

const SPATIAL_BOUNDARY: (crate::ops::Stage, i32) = (crate::ops::Stage::Tone, -35);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum OpRange {
    All,
    BelowBoundary,
    FromBoundary,
}

impl OpRange {
    fn contains(self, op: &dyn crate::ops::Op) -> bool {
        let below = (op.stage(), op.order()) < SPATIAL_BOUNDARY;
        match self {
            Self::All => true,
            Self::BelowBoundary => below,
            Self::FromBoundary => !below,
        }
    }
}

fn downsample(image: &mut LinearImage, new_w: u32, new_h: u32) {
    let (rgb, w, h) = transform::resize_owned_to(
        std::mem::take(&mut image.rgb),
        image.width,
        image.height,
        new_w,
        new_h,
    );
    *image = LinearImage::new(rgb, w, h);
}

pub fn run_pipeline_ops(
    image: &mut LinearImage,
    ctx: &OpContext,
    edits: &Edits,
    rasters: &crate::mask_raster::RasterMap,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<()> {
    run_pipeline_ops_inner(image, ctx, edits, rasters, OpRange::All, None, cancel).map(|_| ())
}

pub(super) fn run_pipeline_ops_inner(
    image: &mut LinearImage,
    ctx: &OpContext,
    edits: &Edits,
    rasters: &crate::mask_raster::RasterMap,
    range: OpRange,
    preview_dims: Option<(u32, u32)>,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<Option<LinearImage>> {
    if let crate::frame::PreviewMode::MaskWeight { layer_id } = &ctx.render.preview_mode {
        let layer = edits.masks.iter().find(|l| &l.id == layer_id);
        let eval = match layer {
            Some(l) => crate::cpu::masked::build_layer_eval(l, rasters),
            None => crate::cpu::masked::LayerEval {
                amount: 0.0,
                invert: false,
                components: Vec::new(),
            },
        };
        let mut global_edits = edits.clone();
        global_edits.masks.clear();
        global_edits.geometry = Default::default();
        let global_ctx = OpContext {
            render: RenderContext {
                wb_coeffs: ctx.render.wb_coeffs,
                cam_to_srgb: ctx.render.cam_to_srgb,
                is_raw: ctx.render.is_raw,
                capture_sigma: ctx.render.capture_sigma,
                preview_mode: crate::frame::PreviewMode::None,
                roi: ctx.render.roi,
                dcp: ctx.render.dcp.clone(),
            },
            scratch: OpScratch::default(),
        };
        run_pipeline_ops(image, &global_ctx, &global_edits, rasters, cancel)?;
        run_output_ops(image, &global_ctx, &global_edits, cancel)?;
        let warp = LensWarpParams::from_edits(&edits.lens, image.width as u32, image.height as u32);
        crate::cpu::masked::render_mask_overlay(image, &eval, &warp, ctx.render.dcp.as_deref());
        let registry = default_registry();
        for op in registry.ops().iter() {
            cancel::check(cancel)?;
            if op.stage() != crate::ops::Stage::Geometry {
                continue;
            }
            if !op.is_active(edits) {
                continue;
            }
            op.apply_cpu(image, ctx, edits)?;
        }
        return Ok(None);
    }
    let registry = default_registry();
    let layer_evals = build_layer_evals(&edits.masks, rasters);
    let layer_edits: Vec<Edits> = edits
        .masks
        .iter()
        .filter(|l| l.is_effective())
        .map(|l| effective_edits_for_layer(edits, l))
        .collect();
    let n_layers = layer_evals.len();
    let presence_active = has_presence(edits) || layer_edits.iter().any(has_presence);
    let shadows_active =
        edits.tone.shadows != 0.0 || layer_edits.iter().any(|e| e.tone.shadows != 0.0);
    let mut pyramid_cache: Option<LumaPyramid> = None;
    let mut pyramid_mips: Option<crate::presence::PresenceMips> = None;
    let ctx_outer = ctx;
    let mut ctx_local: Option<OpContext> = None;
    let mut presence_done = false;
    let mut spatial_ready = false;
    let mut sharpen_delta: Option<LinearImage> = None;
    let sharpen_deltas: Vec<f32> = edits
        .masks
        .iter()
        .filter(|l| l.is_effective())
        .map(|l| l.edits.sharpen.unwrap_or(0.0) as f32)
        .collect();
    let mut segment = FusedSegment::default();
    let mut layer_segments: Vec<FusedSegment> =
        (0..n_layers).map(|_| FusedSegment::default()).collect();
    let mut layer_images: Vec<LinearImage> = Vec::new();
    let mut lens_warp =
        LensWarpParams::from_edits(&edits.lens, image.width as u32, image.height as u32);
    let flush = |image: &mut LinearImage,
                 layer_images: &mut Vec<LinearImage>,
                 segment: &mut FusedSegment,
                 layer_segments: &mut [FusedSegment]| {
        let layers_pending = layer_segments.iter().any(|s| !s.is_empty());
        if segment.is_empty() && !layers_pending {
            return;
        }
        if layers_pending && layer_images.is_empty() {
            layer_images.extend(
                (0..n_layers)
                    .map(|_| LinearImage::new(image.rgb.clone(), image.width, image.height)),
            );
        }
        if !segment.is_empty() {
            apply_segment(image, segment);
            segment.clear();
        }
        for (i, s) in layer_segments.iter_mut().enumerate() {
            if s.is_empty() {
                continue;
            }
            apply_segment(&mut layer_images[i], s);
            s.clear();
        }
    };
    let op_active = |op: &dyn crate::ops::Op| -> bool {
        op.is_active(edits) || layer_edits.iter().any(|e| op.is_active(e))
    };
    for op in registry.ops().iter() {
        cancel::check(cancel)?;
        if !range.contains(op.as_ref()) {
            continue;
        }
        if !op_active(op.as_ref()) {
            continue;
        }
        if op.stage() == crate::ops::Stage::Output {
            continue;
        }
        if op.stage() == crate::ops::Stage::Sensor {
            continue;
        }
        if !spatial_ready && (op.stage(), op.order()) >= SPATIAL_BOUNDARY {
            spatial_ready = true;
            flush(image, &mut layer_images, &mut segment, &mut layer_segments);
            if let Some((pw, ph)) = preview_dims {
                downsample(image, pw, ph);
                for li in layer_images.iter_mut() {
                    downsample(li, pw, ph);
                }
                lens_warp = LensWarpParams::from_edits(
                    &edits.lens,
                    image.width as u32,
                    image.height as u32,
                );
            }
            if shadows_active {
                let w = image.width as u32;
                let h = image.height as u32;
                let radii = presence_radii(w, h);
                let mips = presence_mips(w, h, radii);
                let levels = presence_pyramid_levels(w, h, radii) as usize;
                let pyr = LumaPyramid::build(image, levels);
                let shadows_blur = Arc::new(pyr.upsample(mips.shadows, image.width, image.height));
                pyramid_cache = Some(pyr);
                pyramid_mips = Some(mips);
                ctx_local = Some(OpContext {
                    render: ctx_outer.render.clone(),
                    scratch: OpScratch {
                        shadows_blur: Some(shadows_blur),
                        sharpen_delta: None,
                    },
                });
            }
        }
        let ctx: &OpContext = ctx_local.as_ref().unwrap_or(ctx_outer);
        if op.gpu_route() == GpuRoute::Presence {
            if !presence_done && presence_active {
                flush(image, &mut layer_images, &mut segment, &mut layer_segments);
                let amounts = presence_amounts(edits);
                let layer_amounts: Vec<crate::presence::PresenceAmounts> =
                    layer_edits.iter().map(presence_amounts).collect();
                let w = image.width as u32;
                let h = image.height as u32;
                let radii = presence_radii(w, h);
                let mips = pyramid_mips.unwrap_or_else(|| presence_mips(w, h, radii));
                let pyramid = match pyramid_cache.take() {
                    Some(p) => p,
                    None => {
                        let levels = presence_pyramid_levels(w, h, radii) as usize;
                        LumaPyramid::build(image, levels)
                    }
                };
                let iw = image.width;
                let ih = image.height;
                let needs = |pick: fn(&crate::presence::PresenceAmounts) -> f32| {
                    pick(&amounts) != 0.0 || layer_amounts.iter().any(|a| pick(a) != 0.0)
                };
                let texture_blur =
                    needs(|a| a.texture).then(|| Arc::new(pyramid.upsample(mips.texture, iw, ih)));
                let clarity_blur =
                    needs(|a| a.clarity).then(|| Arc::new(pyramid.upsample(mips.clarity, iw, ih)));
                drop(pyramid);
                let make_op = |a: &crate::presence::PresenceAmounts| CpuFusedOp::Presence {
                    texture: a.texture,
                    clarity: a.clarity,
                    texture_blur: texture_blur.clone(),
                    clarity_blur: clarity_blur.clone(),
                };
                if !amounts.is_zero() {
                    segment.push(make_op(&amounts));
                }
                for (i, s) in layer_segments.iter_mut().enumerate() {
                    if layer_amounts[i].is_zero() {
                        continue;
                    }
                    s.push(make_op(&layer_amounts[i]));
                }
                presence_done = true;
            }
            continue;
        }
        if let Some(fused) = op.cpu_fused(edits, ctx) {
            segment.push(fused);
            for (i, s) in layer_segments.iter_mut().enumerate() {
                if let Some(fl) = op.cpu_fused(&layer_edits[i], ctx) {
                    s.push(fl);
                }
            }
            continue;
        }
        flush(image, &mut layer_images, &mut segment, &mut layer_segments);
        if op.stage() == crate::ops::Stage::Geometry {
            if sharpen_delta.is_none() && sharpen_deltas.iter().any(|d| *d != 0.0) {
                sharpen_delta = Some(crate::cpu::masked::build_sharpen_delta_image(
                    image,
                    &layer_evals,
                    &sharpen_deltas,
                    &lens_warp,
                ));
            }
            if !layer_images.is_empty() {
                blend_layer_images(image, &layer_images, &layer_evals, &lens_warp);
                layer_images.clear();
            }
        }
        op.apply_cpu(image, ctx, edits)?;
        for (i, li) in layer_images.iter_mut().enumerate() {
            op.apply_cpu(li, ctx, &layer_edits[i])?;
        }
        if let Some(d) = sharpen_delta.as_mut()
            && op.stage() == crate::ops::Stage::Geometry
        {
            op.apply_cpu(d, ctx, edits)?;
        }
    }
    flush(image, &mut layer_images, &mut segment, &mut layer_segments);
    if !layer_images.is_empty() {
        blend_layer_images(image, &layer_images, &layer_evals, &lens_warp);
    }
    if !spatial_ready && let Some((pw, ph)) = preview_dims {
        downsample(image, pw, ph);
    }
    if range != OpRange::BelowBoundary
        && sharpen_delta.is_none()
        && sharpen_deltas.iter().any(|d| *d != 0.0)
    {
        sharpen_delta = Some(crate::cpu::masked::build_sharpen_delta_image(
            image,
            &layer_evals,
            &sharpen_deltas,
            &lens_warp,
        ));
    }
    Ok(sharpen_delta)
}

pub fn run_output_ops(
    image: &mut LinearImage,
    ctx: &OpContext,
    edits: &Edits,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<()> {
    let registry = default_registry();
    for op in registry.active(edits) {
        if op.stage() != crate::ops::Stage::Output {
            continue;
        }
        cancel::check(cancel)?;
        op.apply_cpu(image, ctx, edits)?;
    }
    Ok(())
}

pub fn run_sensor_ops(
    image: &mut LinearImage,
    ctx: &OpContext,
    edits: &Edits,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<()> {
    if !edits.lens.any_active() {
        return Ok(());
    }
    let registry = default_registry();
    for op in registry.active(edits) {
        if op.stage() != crate::ops::Stage::Sensor {
            continue;
        }
        cancel::check(cancel)?;
        op.apply_cpu(image, ctx, edits)?;
    }
    Ok(())
}
