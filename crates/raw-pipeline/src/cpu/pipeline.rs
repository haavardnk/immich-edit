use crate::cancel::{self, CancelToken};
use crate::cpu::fused::{CpuFusedOp, FusedSegment, apply_segment};
use crate::cpu::masked::{blend_layer_images, build_layer_evals, effective_edits_for_layer};
use crate::cpu::presence::has_presence;
use crate::cpu::presence_pyramid::LumaPyramid;
use crate::cpu::renderer::{self, CpuRenderer};
use crate::cpu::{demosaic, transform};
use crate::edits::Edits;
use crate::encode::{encode_from_rgb8, encode_from_rgb16};
use crate::frame::{BitDepth, OutputColorSpace, RawFrame, RenderOptions, RenderedImage};
use crate::histogram::{self, Histogram};
use crate::ops::LinearImage;
use crate::ops::lens_distortion::LensWarpParams;
use crate::ops::{GpuOpKind, OpContext, OpScratch, RenderContext, default_registry};
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};
use rayon::prelude::*;
use std::sync::Arc;

type DcpFinish<'a> = (
    Option<&'a crate::dcp::HueSatMap>,
    Option<&'a [[f32; 2]]>,
    &'a [[f32; 3]; 3],
    &'a [[f32; 3]; 3],
);

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

pub fn render(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<RenderedImage> {
    render_with_cancel(frame, edits, options, None)
}

pub fn render_with_cancel(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<RenderedImage> {
    render_cached(frame, edits, options, cancel, None)
}

pub(crate) fn render_cached(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
    cancel: Option<&CancelToken>,
    renderer: Option<&CpuRenderer>,
) -> crate::PipelineResult<RenderedImage> {
    let mut edits = edits.clamped();
    edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(frame.is_raw));
    edits.geometry.crop = crate::geom::compose_roi(edits.geometry.crop, options.roi);

    let src_dims = (frame.width as u32, frame.height as u32);
    let out_dims =
        crate::geom::display_out_dims(frame.orientation, &edits, src_dims, options.max_edge);
    let preview_ratio = crate::geom::preview_ratio(
        frame.orientation,
        &edits,
        src_dims,
        options.max_edge,
        options.quality,
    );

    let setup = crate::dcp_pipeline::resolve(frame, &edits, options.dcp.as_deref());
    let ctx = OpContext {
        render: RenderContext {
            wb_coeffs: frame.wb_coeffs,
            cam_to_srgb: setup.cam_to_srgb,
            is_raw: frame.is_raw,
            capture_sigma: frame.capture_sigma,
            preview_mode: options.preview_mode.clone(),
            roi: options.roi,
            dcp: setup.resolved.clone(),
        },
        scratch: OpScratch::default(),
    };

    let cache_key = renderer
        .filter(|_| renderer::sensor_cacheable(&edits, options))
        .map(|_| renderer::sensor_cache_key(frame, &edits, &setup, options, preview_ratio));

    let cached = cache_key.and_then(|k| renderer.and_then(|r| r.get(k)));

    let (mut image, oriented_w, oriented_h) = match cached {
        Some(stage) => (
            LinearImage::new(stage.rgb.clone(), stage.width, stage.height),
            stage.oriented_w,
            stage.oriented_h,
        ),
        None => {
            let rgb = if frame.cpp == 1 && !frame.cfa_pattern.is_empty() {
                match demosaic::parse_xtrans(&frame.cfa_pattern) {
                    Some(pattern) => {
                        demosaic::xtrans(&frame.data, frame.width, frame.height, &pattern)
                    }
                    None => demosaic::malvar_he_cutler(
                        &frame.data,
                        frame.width,
                        frame.height,
                        &frame.cfa_pattern,
                    ),
                }
            } else {
                frame.data.clone()
            };

            let mut sensor_image = LinearImage::new(rgb, frame.width, frame.height);
            run_sensor_ops(&mut sensor_image, &ctx, &edits, cancel)?;
            cancel::check(cancel)?;
            let (rgb, w, h) = transform::apply_orientation(
                sensor_image.rgb,
                sensor_image.width,
                sensor_image.height,
                frame.orientation,
            );

            let (oriented_w, oriented_h) = match edits.geometry.rotate {
                90 | 270 => (h, w),
                _ => (w, h),
            };

            let mut image = LinearImage::new(rgb, w, h);
            let preview_dims = preview_ratio
                .and_then(|ratio| crate::geom::resample_target((w as u32, h as u32), ratio));

            if let (Some(key), Some(r)) = (cache_key, renderer) {
                run_pipeline_ops_inner(
                    &mut image,
                    &ctx,
                    &edits,
                    &options.rasters,
                    OpRange::BelowBoundary,
                    preview_dims,
                    cancel,
                )?;
                r.put(
                    key,
                    Arc::new(renderer::SensorStage {
                        rgb: image.rgb.clone(),
                        width: image.width,
                        height: image.height,
                        oriented_w,
                        oriented_h,
                    }),
                );
            } else {
                let sharpen_delta = run_pipeline_ops_inner(
                    &mut image,
                    &ctx,
                    &edits,
                    &options.rasters,
                    OpRange::All,
                    preview_dims,
                    cancel,
                )?;
                return finish_render(
                    frame,
                    &edits,
                    options,
                    &ctx,
                    image,
                    sharpen_delta,
                    out_dims,
                    (oriented_w, oriented_h),
                    cancel,
                );
            }
            (image, oriented_w, oriented_h)
        }
    };

    let sharpen_delta = run_pipeline_ops_inner(
        &mut image,
        &ctx,
        &edits,
        &options.rasters,
        OpRange::FromBoundary,
        None,
        cancel,
    )?;

    finish_render(
        frame,
        &edits,
        options,
        &ctx,
        image,
        sharpen_delta,
        out_dims,
        (oriented_w, oriented_h),
        cancel,
    )
}

#[allow(clippy::too_many_arguments)]
fn finish_render(
    frame: &RawFrame,
    edits: &Edits,
    options: &RenderOptions,
    ctx: &OpContext,
    image: LinearImage,
    sharpen_delta: Option<LinearImage>,
    out_dims: (u32, u32),
    oriented: (usize, usize),
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<RenderedImage> {
    let (oriented_w, oriented_h) = oriented;

    cancel::check(cancel)?;
    let (rgb, w, h) =
        transform::resize_owned_to(image.rgb, image.width, image.height, out_dims.0, out_dims.1);

    let mut out_image = LinearImage::new(rgb, w, h);
    let display_ready = matches!(
        options.preview_mode,
        crate::frame::PreviewMode::MaskWeight { .. }
    );
    if !display_ready {
        let out_ctx;
        let ctx: &OpContext = match sharpen_delta {
            Some(d) => {
                out_ctx = OpContext {
                    render: ctx.render.clone(),
                    scratch: OpScratch {
                        shadows_blur: None,
                        sharpen_delta: Some(crate::ops::SharpenDeltaMap {
                            width: d.width,
                            height: d.height,
                            values: Arc::new(d.rgb.iter().step_by(3).copied().collect()),
                        }),
                    },
                };
                &out_ctx
            }
            None => ctx,
        };
        run_output_ops(&mut out_image, ctx, edits, cancel)?;
    }
    let rgb = out_image.rgb;
    let w = out_image.width;
    let h = out_image.height;

    let want_16bit = options.output.bit_depth() == BitDepth::Sixteen;
    cancel::check(cancel)?;
    let lut_resolved = resolve_lut(edits, options)?;
    let lut_ref = lut_resolved.as_ref().map(|(l, a)| (l.as_ref(), *a));
    let dcp_finish = ctx.render.dcp.as_ref().map(|d| {
        (
            d.look_table.as_deref(),
            d.tone_curve.as_deref().map(Vec::as_slice),
            &d.to_pp,
            &d.from_pp,
        )
    });
    let (rgb_u8, rgb_u16, histogram, linear_histogram) = finish_output(
        rgb,
        w,
        h,
        want_16bit,
        display_ready,
        lut_ref,
        dcp_finish,
        options.output_color_space,
        options.gamut_warn,
        options.clip_warn,
    );
    cancel::check(cancel)?;

    let bytes = if want_16bit {
        encode_from_rgb16(
            rgb_u16.as_deref().unwrap(),
            w as u32,
            h as u32,
            &options.output,
            options.output_color_space,
        )?
    } else {
        encode_from_rgb8(
            &rgb_u8,
            w as u32,
            h as u32,
            &options.output,
            options.output_color_space,
        )?
    };

    Ok(RenderedImage {
        bytes,
        histogram,
        linear_histogram: Some(linear_histogram),
        width: w as u32,
        height: h as u32,
        source_w: oriented_w as u32,
        source_h: oriented_h as u32,
        renderer: "cpu".into(),
        is_raw: frame.is_raw,
    })
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

fn run_pipeline_ops_inner(
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
        if op.gpu_kind() == GpuOpKind::Presence {
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

#[cfg(test)]
fn srgb_oetf(v: f32) -> f32 {
    crate::tone::srgb_oetf(v)
}

#[cfg(test)]
fn srgb_oetf_scalar(v: f32) -> f32 {
    crate::tone::srgb_oetf_scalar(v)
}

type HistBins = (
    [u32; histogram::BINS],
    [u32; histogram::BINS],
    [u32; histogram::BINS],
    [u32; histogram::BINS],
);

fn fold_linear(acc: &mut HistBins, lr: f32, lg: f32, lb: f32) {
    let li = (0.2126 * lr + 0.7152 * lg + 0.0722 * lb).clamp(0.0, 1.0);
    acc.0[((lr.clamp(0.0, 1.0) * 255.0) as usize).min(histogram::BINS - 1)] += 1;
    acc.1[((lg.clamp(0.0, 1.0) * 255.0) as usize).min(histogram::BINS - 1)] += 1;
    acc.2[((lb.clamp(0.0, 1.0) * 255.0) as usize).min(histogram::BINS - 1)] += 1;
    acc.3[((li * 255.0) as usize).min(histogram::BINS - 1)] += 1;
}

fn fold_display(acc: &mut HistBins, ur: u8, ug: u8, ub: u8) {
    let li = (0.2126 * ur as f32 + 0.7152 * ug as f32 + 0.0722 * ub as f32) as usize;
    acc.0[ur as usize] += 1;
    acc.1[ug as usize] += 1;
    acc.2[ub as usize] += 1;
    acc.3[li.min(histogram::BINS - 1)] += 1;
}

fn merge_bins(mut a: HistBins, b: HistBins) -> HistBins {
    for i in 0..histogram::BINS {
        a.0[i] += b.0[i];
        a.1[i] += b.1[i];
        a.2[i] += b.2[i];
        a.3[i] += b.3[i];
    }
    a
}

fn bins_to_histogram(bins: HistBins) -> Histogram {
    Histogram {
        r: bins.0.to_vec(),
        g: bins.1.to_vec(),
        b: bins.2.to_vec(),
        l: bins.3.to_vec(),
    }
}

#[inline(always)]
fn dither_hash(x: u32, y: u32, c: u32) -> f32 {
    let mut h =
        x.wrapping_mul(0x8da6_b343) ^ y.wrapping_mul(0xd816_3841) ^ c.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}

#[inline(always)]
fn quantize_u8_dithered(v: f32, x: u32, y: u32, c: u32) -> u8 {
    let tpdf = dither_hash(x, y, c * 2) - dither_hash(x, y, c * 2 + 1);
    ((v.clamp(0.0, 1.0) * 255.0 + tpdf).round()).clamp(0.0, 255.0) as u8
}

#[inline]
fn apply_display_lut(rgb: [f32; 3], lut: Option<(&crate::lut::Lut3d, f32)>) -> [f32; 3] {
    match lut {
        Some((l, amount)) => {
            let sampled = l.sample(rgb);
            [
                rgb[0] + amount * (sampled[0] - rgb[0]),
                rgb[1] + amount * (sampled[1] - rgb[1]),
                rgb[2] + amount * (sampled[2] - rgb[2]),
            ]
        }
        None => rgb,
    }
}

fn resolve_lut(
    edits: &Edits,
    options: &RenderOptions,
) -> crate::PipelineResult<Option<(std::sync::Arc<crate::lut::Lut3d>, f32)>> {
    if !edits.color.lut_3d.is_active() {
        return Ok(None);
    }
    let id = edits
        .color
        .lut_3d
        .lut_id
        .as_ref()
        .ok_or_else(|| crate::PipelineError::Render("lut id missing".into()))?;
    let lut = options
        .luts
        .get(id)
        .ok_or_else(|| crate::PipelineError::Render(format!("lut {id} not loaded")))?;
    Ok(Some((
        lut.clone(),
        (edits.color.lut_3d.amount / 100.0) as f32,
    )))
}

#[allow(clippy::too_many_arguments)]
fn finish_output(
    linear: Vec<f32>,
    w: usize,
    h: usize,
    want_16bit: bool,
    display_ready: bool,
    lut: Option<(&crate::lut::Lut3d, f32)>,
    dcp_finish: Option<DcpFinish>,
    color_space: OutputColorSpace,
    gamut_warn: bool,
    clip_warn: bool,
) -> (Vec<u8>, Option<Vec<u16>>, Histogram, Histogram) {
    let _span = tracing::debug_span!("cpu.finish_output_histogram", w = w, h = h).entered();
    let pixel_count = w * h;
    let n = linear.len();
    let mut rgb_u8 = vec![0u8; n];
    let mut rgb_u16: Vec<u16> = if want_16bit {
        vec![0u16; n]
    } else {
        Vec::new()
    };
    let step = if pixel_count > 500_000 { 2 } else { 1 };
    let chunk_px = histogram::chunk_pixels(pixel_count);
    let chunk = chunk_px * 3;
    let zero = || -> (HistBins, HistBins) {
        (
            (
                [0; histogram::BINS],
                [0; histogram::BINS],
                [0; histogram::BINS],
                [0; histogram::BINS],
            ),
            (
                [0; histogram::BINS],
                [0; histogram::BINS],
                [0; histogram::BINS],
                [0; histogram::BINS],
            ),
        )
    };

    let finalize = |lr: f32, lg: f32, lb: f32| -> ([f32; 3], bool) {
        if display_ready {
            return ([lr, lg, lb], false);
        }
        let finished = match dcp_finish {
            Some((look, curve, to_pp, from_pp)) => {
                crate::color::apply_dcp_finish(look, curve, to_pp, from_pp, [lr, lg, lb])
            }
            None => [lr, lg, lb],
        };
        let clip = gamut_warn && crate::tone::is_out_of_gamut(finished, color_space);
        let display = crate::tone::apply_rgb_cs(finished, color_space);
        (apply_display_lut(display, lut), clip)
    };

    let (lin_bins, dis_bins) = if want_16bit {
        linear
            .par_chunks(chunk)
            .enumerate()
            .zip(rgb_u8.par_chunks_mut(chunk))
            .zip(rgb_u16.par_chunks_mut(chunk))
            .fold(zero, |mut acc, (((ci, s), u8c), u16c)| {
                let base_px = ci * chunk_px;
                let mut i = 0;
                let mut p = 0usize;
                while i + 2 < s.len() {
                    let lr = s[i];
                    let lg = s[i + 1];
                    let lb = s[i + 2];
                    let ([tr, tg, tb], clip) = finalize(lr, lg, lb);
                    let abs_px = base_px + p;
                    let px = (abs_px % w) as u32;
                    let py = (abs_px / w) as u32;
                    let ru = quantize_u8_dithered(tr, px, py, 0);
                    let gu = quantize_u8_dithered(tg, px, py, 1);
                    let bu = quantize_u8_dithered(tb, px, py, 2);
                    u8c[i] = ru;
                    u8c[i + 1] = gu;
                    u8c[i + 2] = bu;
                    u16c[i] = (tr.clamp(0.0, 1.0) * 65535.0) as u16;
                    u16c[i + 1] = (tg.clamp(0.0, 1.0) * 65535.0) as u16;
                    u16c[i + 2] = (tb.clamp(0.0, 1.0) * 65535.0) as u16;
                    if p % step == 0 {
                        fold_linear(&mut acc.0, lr, lg, lb);
                        fold_display(&mut acc.1, ru, gu, bu);
                    }
                    if let Some(paint) = crate::warn::classify([tr, tg, tb], clip, clip_warn) {
                        u8c[i] = paint[0];
                        u8c[i + 1] = paint[1];
                        u8c[i + 2] = paint[2];
                    }
                    i += 3;
                    p += 1;
                }
                acc
            })
            .reduce(zero, |a, b| (merge_bins(a.0, b.0), merge_bins(a.1, b.1)))
    } else {
        linear
            .par_chunks(chunk)
            .enumerate()
            .zip(rgb_u8.par_chunks_mut(chunk))
            .fold(zero, |mut acc, ((ci, s), u8c)| {
                let base_px = ci * chunk_px;
                let mut i = 0;
                let mut p = 0usize;
                while i + 2 < s.len() {
                    let lr = s[i];
                    let lg = s[i + 1];
                    let lb = s[i + 2];
                    let ([tr, tg, tb], clip) = finalize(lr, lg, lb);
                    let abs_px = base_px + p;
                    let px = (abs_px % w) as u32;
                    let py = (abs_px / w) as u32;
                    let ru = quantize_u8_dithered(tr, px, py, 0);
                    let gu = quantize_u8_dithered(tg, px, py, 1);
                    let bu = quantize_u8_dithered(tb, px, py, 2);
                    u8c[i] = ru;
                    u8c[i + 1] = gu;
                    u8c[i + 2] = bu;
                    if p % step == 0 {
                        fold_linear(&mut acc.0, lr, lg, lb);
                        fold_display(&mut acc.1, ru, gu, bu);
                    }
                    if let Some(paint) = crate::warn::classify([tr, tg, tb], clip, clip_warn) {
                        u8c[i] = paint[0];
                        u8c[i + 1] = paint[1];
                        u8c[i + 2] = paint[2];
                    }
                    i += 3;
                    p += 1;
                }
                acc
            })
            .reduce(zero, |a, b| (merge_bins(a.0, b.0), merge_bins(a.1, b.1)))
    };

    let rgb_u16 = if want_16bit { Some(rgb_u16) } else { None };
    (
        rgb_u8,
        rgb_u16,
        bins_to_histogram(dis_bins),
        bins_to_histogram(lin_bins),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_ready_output_skips_tone_mapping() {
        let (rgb, _, _, _) = finish_output(
            vec![0.5, 0.5, 0.5],
            1,
            1,
            false,
            true,
            None,
            None,
            OutputColorSpace::SRgb,
            false,
            false,
        );
        if rgb.iter().any(|value| !(126..=129).contains(value)) {
            panic!("expected display-ready midpoint, got {rgb:?}");
        }
    }

    #[test]
    fn gamut_warn_paints_out_of_gamut_pixels() {
        let (rgb, _, _, _) = finish_output(
            vec![0.9, -0.1, 0.2],
            1,
            1,
            false,
            false,
            None,
            None,
            OutputColorSpace::SRgb,
            true,
            false,
        );
        if rgb != vec![255, 0, 255] {
            panic!("expected magenta gamut warning, got {rgb:?}");
        }
    }

    #[test]
    fn gamut_warn_ignores_bright_in_gamut_pixels() {
        let (rgb, _, _, _) = finish_output(
            vec![4.0, 0.0, 0.0],
            1,
            1,
            false,
            false,
            None,
            None,
            OutputColorSpace::SRgb,
            true,
            false,
        );
        if rgb == vec![255, 0, 255] {
            panic!("bright in-gamut red must not be flagged out of gamut");
        }
    }

    #[test]
    fn gamut_warn_leaves_in_gamut_pixels() {
        let (rgb, _, _, _) = finish_output(
            vec![0.5, 0.5, 0.5],
            1,
            1,
            false,
            false,
            None,
            None,
            OutputColorSpace::SRgb,
            true,
            false,
        );
        if rgb == vec![255, 0, 255] {
            panic!("neutral gray must not be flagged out of gamut");
        }
    }

    #[test]
    fn clip_warn_paints_blown_and_crushed_pixels() {
        let cases = [
            ([4.0f32, 4.0, 4.0], Some(crate::warn::HIGHLIGHT_WARN_RGB)),
            ([0.05, 4.0, 0.05], Some(crate::warn::HIGHLIGHT_WARN_RGB)),
            ([0.0, 0.0, 0.0], Some(crate::warn::SHADOW_WARN_RGB)),
            ([0.5, 0.5, 0.5], None),
            ([0.2, 0.25, 0.22], None),
        ];
        for (linear, want) in cases {
            let (rgb, _, _, _) = finish_output(
                linear.to_vec(),
                1,
                1,
                false,
                false,
                None,
                None,
                OutputColorSpace::SRgb,
                false,
                true,
            );
            match want {
                Some(paint) => {
                    if rgb != paint.to_vec() {
                        panic!("clip warning for {linear:?} = {rgb:?}, want {paint:?}");
                    }
                }
                None => {
                    if rgb == crate::warn::HIGHLIGHT_WARN_RGB.to_vec()
                        || rgb == crate::warn::SHADOW_WARN_RGB.to_vec()
                    {
                        panic!("midtone {linear:?} must not be flagged, got {rgb:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn clip_warn_off_leaves_blown_pixels_alone() {
        let (rgb, _, _, _) = finish_output(
            vec![4.0, 4.0, 4.0],
            1,
            1,
            false,
            false,
            None,
            None,
            OutputColorSpace::SRgb,
            false,
            false,
        );
        if rgb != vec![255, 255, 255] {
            panic!("expected untouched white, got {rgb:?}");
        }
    }

    #[test]
    fn oetf_lut_matches_scalar_for_u8() {
        let mut x = 0.0f32;
        let mut max_err: f32 = 0.0;
        let mut max_q_diff: i32 = 0;
        while x <= 1.0 {
            let lut = srgb_oetf(x);
            let exact = srgb_oetf_scalar(x);
            let err = (lut - exact).abs();
            if err > max_err {
                max_err = err;
            }
            let q_lut = (lut.clamp(0.0, 1.0) * 255.0).round() as i32;
            let q_exact = (exact.clamp(0.0, 1.0) * 255.0).round() as i32;
            let d = (q_lut - q_exact).abs();
            if d > max_q_diff {
                max_q_diff = d;
            }
            x += 1.0 / 8192.0;
        }
        if max_q_diff > 1 {
            panic!("u8 quantization differs by {max_q_diff} (max abs err {max_err})");
        }
    }

    #[test]
    fn oetf_lut_matches_scalar_for_u16() {
        let mut x = 0.0f32;
        let mut max_diff: i32 = 0;
        while x <= 1.0 {
            let lut = srgb_oetf(x);
            let exact = srgb_oetf_scalar(x);
            let q_lut = (lut.clamp(0.0, 1.0) * 65535.0).round() as i32;
            let q_exact = (exact.clamp(0.0, 1.0) * 65535.0).round() as i32;
            let d = (q_lut - q_exact).abs();
            if d > max_diff {
                max_diff = d;
            }
            x += 1.0 / 16384.0;
        }
        if max_diff > 16 {
            panic!("u16 quantization differs by {max_diff}");
        }
    }

    #[test]
    fn dither_is_deterministic_per_pixel() {
        let a = quantize_u8_dithered(0.5, 12, 7, 1);
        let b = quantize_u8_dithered(0.5, 12, 7, 1);
        if a != b {
            panic!("dither must be deterministic for identical coords, got {a} vs {b}");
        }
    }

    #[test]
    fn dither_stays_within_one_lsb() {
        let v = 100.0 / 255.0;
        let base = (v * 255.0f32).round() as i32;
        let mut x = 0u32;
        while x < 64 {
            let q = quantize_u8_dithered(v, x, 0, 0) as i32;
            if (q - base).abs() > 1 {
                panic!("dither perturbation exceeds 1 LSB at x={x}: {q} vs {base}");
            }
            x += 1;
        }
    }

    #[test]
    fn dither_breaks_bands_on_subtle_gradient() {
        let lo = 100.4 / 255.0;
        let hi = 100.6 / 255.0;
        let mut seen_lo = false;
        let mut seen_hi = false;
        let mut x = 0u32;
        while x < 256 {
            let v = lo + (hi - lo) * (x as f32 / 255.0);
            let q = quantize_u8_dithered(v, x, 3, 2);
            if q == 100 {
                seen_lo = true;
            }
            if q == 101 {
                seen_hi = true;
            }
            x += 1;
        }
        if !(seen_lo && seen_hi) {
            panic!("dither should distribute a sub-LSB gradient across both bins");
        }
    }
}
