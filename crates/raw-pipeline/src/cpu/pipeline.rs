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
use crate::timing::{self, StageClock};
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
    edits.detail.sharpen_amount = Some(edits.detail.sharpen_amount_for(frame.meta.is_raw));
    edits.geometry.crop = crate::geom::compose_roi(edits.geometry.crop, options.roi);

    let src_dims = (frame.meta.width as u32, frame.meta.height as u32);
    let out_dims =
        crate::geom::display_out_dims(frame.meta.orientation, &edits, src_dims, options.max_edge);
    let preview_ratio = crate::geom::preview_ratio(
        frame.meta.orientation,
        &edits,
        src_dims,
        options.max_edge,
        options.quality,
    );
    let block = crate::geom::superpixel_block(frame, options, preview_ratio);
    let block_scale = block.unwrap_or(1) as f32;

    let setup = crate::dcp_pipeline::resolve(&frame.meta, &edits, options.dcp.as_deref());
    let ctx = OpContext {
        render: RenderContext {
            wb_coeffs: frame.meta.wb_coeffs,
            cam_to_srgb: setup.cam_to_srgb,
            is_raw: frame.meta.is_raw,
            capture_sigma: frame.meta.capture_sigma.map(|s| s / block_scale),
            preview_mode: options.preview_mode.clone(),
            roi: options.roi,
            dcp: setup.resolved.clone(),
        },
        scratch: OpScratch::default(),
    };

    let cache_key = renderer
        .filter(|_| renderer::sensor_cacheable(options))
        .map(|_| renderer::sensor_cache_key(frame, &edits, &setup, options, preview_ratio, block));

    let cached = cache_key.and_then(|k| renderer.and_then(|r| r.get(k)));
    let clock = StageClock::default();

    let (mut image, oriented_w, oriented_h) = match cached {
        Some(stage) => (
            LinearImage::new(stage.rgb.clone(), stage.width, stage.height),
            stage.oriented_w,
            stage.oriented_h,
        ),
        None => {
            let (rgb, sensor_w, sensor_h) = clock.time(timing::DEMOSAIC, || match block {
                Some(block) => (
                    demosaic::superpixel(
                        &frame.data,
                        frame.meta.width,
                        frame.meta.height,
                        &frame.cfa_pattern,
                        block,
                    ),
                    frame.meta.width / block,
                    frame.meta.height / block,
                ),
                None => (full_demosaic(frame), frame.meta.width, frame.meta.height),
            });

            let mut sensor_image = LinearImage::new(rgb, sensor_w, sensor_h);
            clock.time(timing::LENS, || {
                run_sensor_ops(&mut sensor_image, &ctx, &edits, cancel)
            })?;
            cancel::check(cancel)?;
            let (rgb, w, h) = transform::apply_orientation(
                sensor_image.rgb,
                sensor_image.width,
                sensor_image.height,
                frame.meta.orientation,
            );

            let full = if frame.meta.orientation.0 {
                (frame.meta.height, frame.meta.width)
            } else {
                (frame.meta.width, frame.meta.height)
            };
            let (oriented_w, oriented_h) = match edits.geometry.rotate {
                90 | 270 => (full.1, full.0),
                _ => full,
            };

            let mut image = LinearImage::new(rgb, w, h);
            let preview_dims = preview_ratio.and_then(|ratio| {
                crate::geom::resample_target((w as u32, h as u32), ratio / block_scale)
            });

            if let (Some(key), Some(r)) = (cache_key, renderer) {
                run_pipeline_ops_inner(
                    &mut image,
                    &ctx,
                    &edits,
                    &options.rasters,
                    OpRange::BelowBoundary,
                    preview_dims,
                    &clock,
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
                    &clock,
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
                    clock,
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
        &clock,
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
        clock,
        cancel,
    )
}

fn full_demosaic(frame: &RawFrame) -> Vec<f32> {
    if frame.cpp != 1 || frame.cfa_pattern.is_empty() {
        return frame.data.clone();
    }
    match demosaic::parse_xtrans(&frame.cfa_pattern) {
        Some(pattern) => {
            demosaic::xtrans(&frame.data, frame.meta.width, frame.meta.height, &pattern)
        }
        None => demosaic::malvar_he_cutler(
            &frame.data,
            frame.meta.width,
            frame.meta.height,
            &frame.cfa_pattern,
        ),
    }
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
    clock: StageClock,
    cancel: Option<&CancelToken>,
) -> crate::PipelineResult<RenderedImage> {
    let (oriented_w, oriented_h) = oriented;

    cancel::check(cancel)?;
    let (rgb, w, h) = clock.time(timing::RESAMPLE, || {
        transform::resize_owned_to(image.rgb, image.width, image.height, out_dims.0, out_dims.1)
    });

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
        clock.time(timing::OUTPUT, || {
            run_output_ops(&mut out_image, ctx, edits, cancel)
        })?;
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
            d.tone_curve.as_deref(),
            &d.to_pp,
            &d.from_pp,
        )
    });
    let (rgb_u8, rgb_u16, histograms) = clock.time(timing::FINISH, || {
        finish_output(
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
            options.histogram,
        )
    });
    cancel::check(cancel)?;
    let (histogram, linear_histogram) = match histograms {
        Some(h) => (Some(h.display), Some(h.linear)),
        None => (None, None),
    };

    let scopes = options.scopes.then(|| {
        clock.time(timing::SCOPES, || {
            crate::scopes::ScopeGrids::from_rgb_u8(&rgb_u8, w, h)
        })
    });

    let bytes = clock.time(timing::ENCODE, || {
        if want_16bit {
            encode_from_rgb16(
                rgb_u16.as_deref().unwrap(),
                w as u32,
                h as u32,
                &options.output,
                options.output_color_space,
            )
        } else {
            encode_from_rgb8(
                &rgb_u8,
                w as u32,
                h as u32,
                &options.output,
                options.output_color_space,
            )
        }
    })?;

    Ok(RenderedImage {
        bytes,
        histogram,
        linear_histogram,
        scopes,
        width: w as u32,
        height: h as u32,
        source_w: oriented_w as u32,
        source_h: oriented_h as u32,
        renderer: "cpu".into(),
        is_raw: frame.meta.is_raw,
        timings: clock.finish(),
    })
}
