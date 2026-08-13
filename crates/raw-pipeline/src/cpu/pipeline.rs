mod bins;
mod ops;
mod output;

use crate::cancel::{self, CancelToken};
use crate::cpu::renderer::{self, CpuRenderer};
use crate::cpu::{demosaic, transform};
use crate::edits::Edits;
use crate::encode::{encode_from_rgb8, encode_from_rgb16};
use crate::frame::{BitDepth, RawFrame, RenderOptions, RenderedImage};
use crate::ops::LinearImage;
use crate::ops::{OpContext, OpScratch, RenderContext};
use ops::{OpRange, run_pipeline_ops_inner};
use output::{finish_output, resolve_lut};
use std::sync::Arc;

pub use ops::{run_output_ops, run_pipeline_ops, run_sensor_ops};

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
