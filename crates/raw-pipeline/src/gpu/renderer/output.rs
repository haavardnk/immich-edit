use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureViewDescriptor};

use super::GpuRenderer;
use super::display::DisplayFrame;
use super::masks::Retained;
use super::meta::MetaCounts;
use crate::PipelineResult;
use crate::encode::{encode_from_rgb16, encode_from_rgba8};
use crate::frame::{RenderOptions, RenderedImage};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::readback::{
    copy_texture_to_buffer, make_readback_buffer_wide, read_rgba8, read_rgba16uint_as_rgb,
};
use crate::gpu::resources::OutputTargets;
use crate::gpu::timer::RenderTimings;
use crate::timing;

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

    pub(super) fn readback_image<'a>(
        &'a self,
        frame: DisplayFrame<'a>,
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<ReadbackImage<'a>> {
        let DisplayFrame {
            mut encoder,
            targets,
            slot,
            meta,
            dims,
            source_dims,
            atlases,
            timings,
            sharpen,
            scratch,
            retained,
        } = frame;
        let (out_w, out_h) = dims;
        let started = std::time::Instant::now();
        let p = &targets[0];
        let display_src = slot.texture(p);
        let wide = (display_src.format() == wgpu::TextureFormat::Rgba16Uint)
            .then(|| make_readback_buffer_wide(&self.ctx.device, "readback-16", out_w, out_h));
        let display_dst = wide.as_ref().unwrap_or(&p.readback);
        copy_texture_to_buffer(&mut encoder, display_src, display_dst, out_w, out_h);
        self.ctx.queue.submit(Some(encoder.finish()));
        drop((sharpen, scratch, retained));

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
        timings
            .clock()
            .add_wall(timing::READBACK, started.elapsed());
        let [preview_atlas, layer_atlas] = atlases;
        self.release_mask_atlas(preview_atlas);
        self.release_mask_atlas(layer_atlas);
        Ok(ReadbackImage {
            display,
            counts,
            dims,
            source_dims,
            timings,
        })
    }
}

pub(super) struct ReadbackImage<'a> {
    display: DisplayBuf,
    counts: MetaCounts,
    dims: (u32, u32),
    source_dims: (u32, u32),
    timings: RenderTimings<'a>,
}

pub(super) fn finish_image(
    image: ReadbackImage<'_>,
    opts: &RenderOptions,
    is_raw: bool,
    cancel: Option<&crate::cancel::CancelToken>,
) -> PipelineResult<RenderedImage> {
    let ReadbackImage {
        display,
        counts,
        dims: (out_w, out_h),
        source_dims,
        timings,
    } = image;
    let clock = timings.clock();
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
    let bytes = bytes?;
    Ok(RenderedImage {
        bytes,
        histogram,
        linear_histogram,
        scopes,
        width: out_w,
        height: out_h,
        source_w: source_dims.0,
        source_h: source_dims.1,
        renderer: "gpu".into(),
        is_raw,
        timings: timings.finish(cancel),
    })
}
