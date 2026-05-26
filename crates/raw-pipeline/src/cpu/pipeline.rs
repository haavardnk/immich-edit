use crate::cancel::{self, CancelToken};
use crate::cpu::fused::{CpuFusedOp, FusedSegment, apply_segment};
use crate::cpu::masked::{
    LayerEval, apply_segment_masked, build_layer_evals, effective_edits_for_layer,
};
use crate::cpu::presence::has_presence;
use crate::cpu::presence_pyramid::LumaPyramid;
use crate::cpu::{demosaic, transform};
use crate::edits::Edits;
use crate::encode::{encode_from_rgb8, encode_from_rgb16};
use crate::frame::{BitDepth, RawFrame, RenderOptions, RenderedImage};
use crate::histogram::{self, Histogram};
use crate::ops::LinearImage;
use crate::ops::lens_distortion::LensWarpParams;
use crate::ops::{GpuOpKind, OpContext, OpScratch, RenderContext, default_registry};
use crate::presence::{presence_amounts, presence_mips, presence_pyramid_levels, presence_radii};
use rayon::prelude::*;
use std::sync::Arc;

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
    let edits = edits.clamped();

    let (rgb, src_w, src_h) = if frame.cpp == 1 && !frame.cfa_pattern.is_empty() {
        let d =
            demosaic::malvar_he_cutler(&frame.data, frame.width, frame.height, &frame.cfa_pattern);
        (d, frame.width, frame.height)
    } else {
        (frame.data.clone(), frame.width, frame.height)
    };

    let xyz_to_cam = if frame.color_matrices.len() >= 2 {
        let cct = crate::color::estimate_scene_cct(
            frame.wb_coeffs,
            &frame.color_matrices.last().unwrap().1,
        );
        crate::color::interpolate_xyz_to_cam(&frame.color_matrices, cct)
    } else {
        frame.xyz_to_cam
    };
    let cam_to_srgb = if frame.is_raw && !crate::color::is_unusable_matrix(&xyz_to_cam) {
        crate::color::cam_to_srgb_matrix(xyz_to_cam)
    } else {
        crate::color::identity_3x3()
    };
    let ctx = OpContext {
        render: RenderContext {
            wb_coeffs: frame.wb_coeffs,
            cam_to_srgb,
            is_raw: frame.is_raw,
            preview_mode: options.preview_mode.clone(),
        },
        scratch: OpScratch { shadows_blur: None },
    };

    let mut sensor_image = LinearImage::new(rgb, src_w, src_h);
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

    run_pipeline_ops(&mut image, &ctx, &edits, &options.rasters, cancel)?;

    cancel::check(cancel)?;
    let (rgb, w, h) =
        transform::resize_owned(image.rgb, image.width, image.height, options.max_edge);

    let mut out_image = LinearImage::new(rgb, w, h);
    run_output_ops(&mut out_image, &ctx, &edits, cancel)?;
    let rgb = out_image.rgb;
    let w = out_image.width;
    let h = out_image.height;

    let want_16bit = options.output.bit_depth() == BitDepth::Sixteen;
    cancel::check(cancel)?;
    let (rgb_u8, rgb_u16, histogram, linear_histogram) =
        finish_output(rgb, w, h, want_16bit, edits.output);
    cancel::check(cancel)?;

    let bytes = if want_16bit {
        encode_from_rgb16(
            rgb_u16.as_deref().unwrap(),
            w as u32,
            h as u32,
            &options.output,
        )?
    } else {
        encode_from_rgb8(&rgb_u8, w as u32, h as u32, &options.output)?
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
    })
}

pub fn run_pipeline_ops(
    image: &mut LinearImage,
    ctx: &OpContext,
    edits: &Edits,
    rasters: &crate::mask_raster::RasterMap,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<()> {
    if let crate::frame::PreviewMode::MaskWeight { layer_id } = &ctx.render.preview_mode {
        let layer = edits.masks.iter().find(|l| &l.id == layer_id);
        let eval = match layer {
            Some(l) => crate::cpu::masked::build_layer_eval(l, rasters),
            None => crate::cpu::masked::LayerEval {
                amount: 0.0,
                components: Vec::new(),
            },
        };
        let warp = LensWarpParams::from_edits(&edits.lens, image.width as u32, image.height as u32);
        crate::cpu::masked::render_mask_weight(image, &eval, &warp);
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
        return Ok(());
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
    let presence_active = has_presence(edits);
    let shadows_active =
        edits.tone.shadows != 0.0 || layer_edits.iter().any(|e| e.tone.shadows != 0.0);
    let mut pyramid_cache: Option<LumaPyramid> = None;
    let mut pyramid_mips: Option<crate::presence::PresenceMips> = None;
    let ctx_local;
    let ctx: &OpContext = if shadows_active {
        let w = image.width as u32;
        let h = image.height as u32;
        let radii = presence_radii(w, h);
        let mips = presence_mips(w, h, radii);
        let levels = presence_pyramid_levels(w, h, radii) as usize;
        let pyr = LumaPyramid::build(image, levels);
        let shadows_blur = Arc::new(pyr.upsample(mips.shadows, image.width, image.height));
        pyramid_cache = Some(pyr);
        pyramid_mips = Some(mips);
        ctx_local = OpContext {
            render: RenderContext {
                wb_coeffs: ctx.render.wb_coeffs,
                cam_to_srgb: ctx.render.cam_to_srgb,
                is_raw: ctx.render.is_raw,
                preview_mode: ctx.render.preview_mode.clone(),
            },
            scratch: OpScratch {
                shadows_blur: Some(shadows_blur),
            },
        };
        &ctx_local
    } else {
        ctx
    };
    let mut presence_done = false;
    let mut segment = FusedSegment::default();
    let mut layer_segments: Vec<FusedSegment> =
        (0..n_layers).map(|_| FusedSegment::default()).collect();
    let lens_warp =
        LensWarpParams::from_edits(&edits.lens, image.width as u32, image.height as u32);
    let flush = |image: &mut LinearImage,
                 segment: &mut FusedSegment,
                 layer_segments: &mut [FusedSegment],
                 layer_evals: &[LayerEval]| {
        if n_layers == 0 {
            if !segment.is_empty() {
                apply_segment(image, segment);
                segment.clear();
            }
        } else if !segment.is_empty() || layer_segments.iter().any(|s| !s.is_empty()) {
            apply_segment_masked(image, segment, layer_segments, layer_evals, &lens_warp);
            segment.clear();
            for s in layer_segments.iter_mut() {
                s.clear();
            }
        }
    };
    let op_active = |op: &crate::ops::AnyOp| -> bool {
        op.is_active(edits) || layer_edits.iter().any(|e| op.is_active(e))
    };
    for op in registry.ops().iter() {
        cancel::check(cancel)?;
        if !op_active(op) {
            continue;
        }
        if op.stage() == crate::ops::Stage::Output {
            continue;
        }
        if op.stage() == crate::ops::Stage::Sensor {
            continue;
        }
        if op.gpu_kind() == GpuOpKind::Detail {
            continue;
        }
        if op.gpu_kind() == GpuOpKind::Presence {
            if !presence_done && presence_active {
                flush(image, &mut segment, &mut layer_segments, &layer_evals);
                let amounts = presence_amounts(edits);
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
                let texture_blur = (amounts.texture != 0.0)
                    .then(|| Arc::new(pyramid.upsample(mips.texture, iw, ih)));
                let clarity_blur = (amounts.clarity != 0.0)
                    .then(|| Arc::new(pyramid.upsample(mips.clarity, iw, ih)));
                let dehaze_blur = (amounts.dehaze != 0.0)
                    .then(|| Arc::new(pyramid.upsample(mips.dehaze, iw, ih)));
                drop(pyramid);
                let presence_op = CpuFusedOp::Presence {
                    texture: amounts.texture,
                    clarity: amounts.clarity,
                    dehaze: amounts.dehaze,
                    texture_blur,
                    clarity_blur,
                    dehaze_blur,
                };
                segment.push(presence_op.clone());
                for s in layer_segments.iter_mut() {
                    s.push(presence_op.clone());
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
        flush(image, &mut segment, &mut layer_segments, &layer_evals);
        op.apply_cpu(image, ctx, edits)?;
    }
    flush(image, &mut segment, &mut layer_segments, &layer_evals);
    Ok(())
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
pub(crate) fn default_tone(v: f32) -> f32 {
    crate::tone::default_scalar(v)
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

fn finish_output(
    linear: Vec<f32>,
    w: usize,
    h: usize,
    want_16bit: bool,
    output: crate::edits::OutputEdits,
) -> (Vec<u8>, Option<Vec<u16>>, Histogram, Histogram) {
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

    let (lin_bins, dis_bins) = if want_16bit {
        linear
            .par_chunks(chunk)
            .zip(rgb_u8.par_chunks_mut(chunk))
            .zip(rgb_u16.par_chunks_mut(chunk))
            .fold(zero, |mut acc, ((s, u8c), u16c)| {
                let mut i = 0;
                let mut p = 0usize;
                while i + 2 < s.len() {
                    let lr = s[i];
                    let lg = s[i + 1];
                    let lb = s[i + 2];
                    let [tr, tg, tb] = crate::tone::apply_rgb([lr, lg, lb], output);
                    let ru = (tr.clamp(0.0, 1.0) * 255.0) as u8;
                    let gu = (tg.clamp(0.0, 1.0) * 255.0) as u8;
                    let bu = (tb.clamp(0.0, 1.0) * 255.0) as u8;
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
                    i += 3;
                    p += 1;
                }
                acc
            })
            .reduce(zero, |a, b| (merge_bins(a.0, b.0), merge_bins(a.1, b.1)))
    } else {
        linear
            .par_chunks(chunk)
            .zip(rgb_u8.par_chunks_mut(chunk))
            .fold(zero, |mut acc, (s, u8c)| {
                let mut i = 0;
                let mut p = 0usize;
                while i + 2 < s.len() {
                    let lr = s[i];
                    let lg = s[i + 1];
                    let lb = s[i + 2];
                    let [tr, tg, tb] = crate::tone::apply_rgb([lr, lg, lb], output);
                    let ru = (tr.clamp(0.0, 1.0) * 255.0) as u8;
                    let gu = (tg.clamp(0.0, 1.0) * 255.0) as u8;
                    let bu = (tb.clamp(0.0, 1.0) * 255.0) as u8;
                    u8c[i] = ru;
                    u8c[i + 1] = gu;
                    u8c[i + 2] = bu;
                    if p % step == 0 {
                        fold_linear(&mut acc.0, lr, lg, lb);
                        fold_display(&mut acc.1, ru, gu, bu);
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
    fn default_tone_preserves_endpoints() {
        if default_tone(0.0).abs() > 1e-4 {
            panic!("expected 0 at 0");
        }
        let one = default_tone(1.0);
        if !(0.97..=1.0).contains(&one) {
            panic!("expected ~1 at 1, got {one}");
        }
    }

    #[test]
    fn highlight_headroom_softly_compresses() {
        let at_one = default_tone(1.0);
        let above = default_tone(1.5);
        let far = default_tone(4.0);
        if above <= at_one {
            panic!("expected monotonic above 1.0: {at_one} -> {above}");
        }
        if above >= 1.0 || far >= 1.0 {
            panic!("expected soft-clip below 1.0: above={above} far={far}");
        }
        if far <= above {
            panic!("expected monotonic far above 1.0: {above} -> {far}");
        }
    }

    #[test]
    fn default_tone_bounded() {
        let mut x = -0.5f32;
        while x < 3.0 {
            let y = default_tone(x);
            if !(0.0..=1.0).contains(&y) {
                panic!("out of bounds at x={x}: {y}");
            }
            x += 0.05;
        }
    }

    #[test]
    fn default_tone_monotonic() {
        let mut prev = default_tone(0.0);
        let mut x = 0.01f32;
        while x <= 1.0 {
            let y = default_tone(x);
            if y < prev - 1e-5 {
                panic!("non-monotonic at x={x}: {prev} -> {y}");
            }
            prev = y;
            x += 0.01;
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
}
