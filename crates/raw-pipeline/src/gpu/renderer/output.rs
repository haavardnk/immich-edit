use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureViewDescriptor};

use super::GpuRenderer;
use super::masks::Retained;
use super::meta::{MetaCounts, MetaRequest};
use crate::PipelineResult;
use crate::encode::{encode_from_rgb16, encode_from_rgba8};
use crate::frame::{RenderOptions, RenderedImage};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::readback::{copy_texture_to_buffer, read_rgba8, read_rgba16uint_as_rgb};
use crate::gpu::resources::OutputTargets;
use crate::gpu::timer::RenderTimings;
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
        meta: MetaRequest,
        out_dims: (u32, u32),
        t: &RenderTimings,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<(DisplayBuf, MetaCounts)> {
        let (out_w, out_h) = out_dims;
        let started = std::time::Instant::now();
        let display_dst = display_dst.unwrap_or(&p.readback);
        copy_texture_to_buffer(&mut encoder, display_src, display_dst, out_w, out_h);
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
        let counts = self.read_meta_counts(p, meta, cancel)?;
        t.clock().add_wall(timing::READBACK, started.elapsed());
        Ok((display, counts))
    }
}

pub(super) fn finish_image(
    display: DisplayBuf,
    counts: MetaCounts,
    out_dims: (u32, u32),
    source: (u32, u32),
    opts: &RenderOptions,
    is_raw: bool,
    clock: &StageClock,
) -> PipelineResult<RenderedImage> {
    let (out_w, out_h) = out_dims;
    let display = match display {
        DisplayBuf::Rgba8(mut rgba) if opts.gamut_warn || opts.clip_warn => {
            crate::warn::paint_rgba8(&mut rgba, opts.gamut_warn, opts.clip_warn);
            DisplayBuf::Rgba8(rgba)
        }
        other => other,
    };
    let (histogram, linear_histogram) = counts.histograms();
    let (scopes, bytes) = rayon::join(
        || counts.scopes(clock),
        || {
            clock.time(timing::ENCODE, || match &display {
                DisplayBuf::Rgb16(rgb) => {
                    encode_from_rgb16(rgb, out_w, out_h, &opts.output, opts.output_color_space)
                }
                DisplayBuf::Rgba8(rgba) => {
                    encode_from_rgba8(rgba, out_w, out_h, &opts.output, opts.output_color_space)
                }
            })
        },
    );

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
