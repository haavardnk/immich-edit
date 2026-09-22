use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureViewDescriptor};

use super::GpuRenderer;
use super::masks::Retained;
use crate::PipelineResult;
use crate::encode::{encode_from_rgb16, encode_from_rgba8};
use crate::frame::{RenderOptions, RenderedImage};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::readback::{
    copy_texture_to_buffer, read_rgba8, read_rgba16f_as_rgb, read_rgba16uint_as_rgb,
};
use crate::gpu::resources::OutputTargets;
use crate::gpu::timer::RenderTimings;
use crate::histogram::Histogram;
use crate::scopes::ScopeGrids;
use crate::timing::{self, StageClock};

pub(super) enum DisplayBuf {
    Rgba8(Vec<u8>),
    Rgb16(Vec<u16>),
}

impl GpuRenderer {
    pub(super) fn encode_mask_overlay(
        &self,
        encoder: &mut CommandEncoder,
        p: &OutputTargets,
        display_src: &Texture,
        out_dims: (u32, u32),
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let params = crate::gpu::passes::mask_overlay::pack_params(
            out_dims.0,
            out_dims.1,
            crate::gpu::passes::mask_overlay::OVERLAY_ALPHA,
        );
        let params_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mask-overlay-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        let src_view = display_src.create_view(&TextureViewDescriptor::default());
        let weight_view = p.mask_weight.create_view(&TextureViewDescriptor::default());
        let dst_view = p
            .mask_scratch_tone
            .create_view(&TextureViewDescriptor::default());
        let bind = bind_group(
            device,
            "mask-overlay-bg",
            &self.passes.mask_overlay.layout,
            &[
                params_buf.as_entire_binding(),
                tex(&src_view),
                tex(&weight_view),
                tex(&dst_view),
            ],
        );
        dispatch_2d(
            encoder,
            "mask-overlay",
            &self.passes.mask_overlay.pipeline,
            &bind,
            out_dims.0.div_ceil(16),
            out_dims.1.div_ceil(16),
        );
        retained.bufs.push(params_buf);
        retained.binds.push(bind);
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn readback_image(
        &self,
        mut encoder: CommandEncoder,
        p: &OutputTargets,
        display_src: &Texture,
        display_dst: Option<&wgpu::Buffer>,
        linear_src: Option<&Texture>,
        out_dims: (u32, u32),
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<(DisplayBuf, Option<Vec<f32>>)> {
        let (out_w, out_h) = out_dims;
        let started = std::time::Instant::now();
        let display_dst = display_dst.unwrap_or(&p.readback);
        copy_texture_to_buffer(&mut encoder, display_src, display_dst, out_w, out_h);
        if let Some(src) = linear_src {
            copy_texture_to_buffer(&mut encoder, src, &p.linear_readback, out_w, out_h);
        }
        self.ctx.queue.submit(Some(encoder.finish()));

        let display = match display_src.format() {
            wgpu::TextureFormat::Rgba16Uint => DisplayBuf::Rgb16(read_rgba16uint_as_rgb(
                &self.ctx,
                display_dst,
                out_w,
                out_h,
                cancel,
            )?),
            _ => DisplayBuf::Rgba8(read_rgba8(&self.ctx, display_dst, out_w, out_h, cancel)?),
        };
        let linear_rgb = linear_src
            .map(|_| read_rgba16f_as_rgb(&self.ctx, &p.linear_readback, out_w, out_h, cancel))
            .transpose()?;
        t.clock().add_wall(timing::READBACK, started.elapsed());
        Ok((display, linear_rgb))
    }
}

pub(super) fn finish_image(
    display: DisplayBuf,
    linear_rgb: Option<Vec<f32>>,
    out_dims: (u32, u32),
    source: (u32, u32),
    opts: &RenderOptions,
    is_raw: bool,
    clock: &StageClock,
) -> PipelineResult<RenderedImage> {
    let (out_w, out_h) = out_dims;
    let mut rgb16: Option<Vec<u16>> = None;
    let mut rgba = match display {
        DisplayBuf::Rgba8(rgba) => rgba,
        DisplayBuf::Rgb16(rgb) => {
            let need_meta = linear_rgb.is_some() || opts.scopes;
            let rgba = if need_meta {
                rgb.chunks_exact(3)
                    .flat_map(|px| {
                        [
                            (px[0] >> 8) as u8,
                            (px[1] >> 8) as u8,
                            (px[2] >> 8) as u8,
                            255,
                        ]
                    })
                    .collect()
            } else {
                Vec::new()
            };
            rgb16 = Some(rgb);
            rgba
        }
    };
    if rgb16.is_none() && (opts.gamut_warn || opts.clip_warn) {
        crate::warn::paint_rgba8(&mut rgba, opts.gamut_warn, opts.clip_warn);
    }

    let ((histograms, scopes), bytes) = rayon::join(
        || {
            rayon::join(
                || {
                    linear_rgb.map(|linear| {
                        let _span =
                            tracing::debug_span!("gpu.histogram", w = out_w, h = out_h).entered();
                        clock.time(timing::HISTOGRAM, || {
                            rayon::join(
                                || Histogram::from_rgba8(&rgba),
                                || Histogram::from_rgb(&linear, out_w as usize, out_h as usize),
                            )
                        })
                    })
                },
                || {
                    opts.scopes.then(|| {
                        let _s = tracing::debug_span!("gpu.scopes", w = out_w, h = out_h).entered();
                        clock.time(timing::SCOPES, || {
                            ScopeGrids::from_rgba8(&rgba, out_w as usize, out_h as usize)
                        })
                    })
                },
            )
        },
        || {
            clock.time(timing::ENCODE, || match &rgb16 {
                Some(rgb) => {
                    encode_from_rgb16(rgb, out_w, out_h, &opts.output, opts.output_color_space)
                }
                None => {
                    encode_from_rgba8(&rgba, out_w, out_h, &opts.output, opts.output_color_space)
                }
            })
        },
    );
    let (histogram, linear_histogram) = match histograms {
        Some((display, linear)) => (Some(display), Some(linear)),
        None => (None, None),
    };

    Ok(RenderedImage {
        bytes: bytes?,
        histogram,
        linear_histogram,
        scopes,
        width: out_w,
        height: out_h,
        source_w: source.0,
        source_h: source.1,
        renderer: "gpu".into(),
        is_raw,
        timings: Vec::new(),
    })
}
