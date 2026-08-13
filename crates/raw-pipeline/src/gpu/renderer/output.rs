use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureViewDescriptor};

use super::GpuRenderer;
use super::masks::Retained;
use crate::PipelineResult;
use crate::encode::encode_from_rgba8;
use crate::frame::{RenderOptions, RenderedImage};
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::readback::{copy_texture_to_buffer, read_rgba8, read_rgba16f_as_rgb};
use crate::gpu::resources::OutputTargets;
use crate::histogram::Histogram;

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

    pub(super) fn readback_image(
        &self,
        mut encoder: CommandEncoder,
        p: &OutputTargets,
        display_src: &Texture,
        linear_src: &Texture,
        out_dims: (u32, u32),
        cancel: Option<&crate::cancel::CancelToken>,
    ) -> PipelineResult<(Vec<u8>, Vec<f32>)> {
        let (out_w, out_h) = out_dims;
        copy_texture_to_buffer(&mut encoder, display_src, &p.readback, out_w, out_h);
        copy_texture_to_buffer(&mut encoder, linear_src, &p.linear_readback, out_w, out_h);
        self.ctx.queue.submit(Some(encoder.finish()));

        let rgba = read_rgba8(&self.ctx, &p.readback, out_w, out_h, cancel)?;
        let linear_rgb = read_rgba16f_as_rgb(&self.ctx, &p.linear_readback, out_w, out_h, cancel)?;
        Ok((rgba, linear_rgb))
    }
}

pub(super) fn finish_image(
    mut rgba: Vec<u8>,
    linear_rgb: Vec<f32>,
    out_dims: (u32, u32),
    source: (u32, u32),
    opts: &RenderOptions,
    is_raw: bool,
) -> PipelineResult<RenderedImage> {
    let (out_w, out_h) = out_dims;
    if opts.gamut_warn || opts.clip_warn {
        crate::warn::paint_rgba8(&mut rgba, opts.gamut_warn, opts.clip_warn);
    }

    let ((histogram, linear_histogram), bytes) = rayon::join(
        || {
            let _span = tracing::debug_span!("gpu.histogram", w = out_w, h = out_h).entered();
            rayon::join(
                || {
                    let _s = tracing::debug_span!("gpu.histogram.display", w = out_w, h = out_h)
                        .entered();
                    Histogram::from_rgba8(&rgba)
                },
                || {
                    let _s = tracing::debug_span!("gpu.histogram.linear", w = out_w, h = out_h)
                        .entered();
                    Histogram::from_rgb(&linear_rgb, out_w as usize, out_h as usize)
                },
            )
        },
        || encode_from_rgba8(&rgba, out_w, out_h, &opts.output, opts.output_color_space),
    );

    Ok(RenderedImage {
        bytes: bytes?,
        histogram,
        linear_histogram: Some(linear_histogram),
        width: out_w,
        height: out_h,
        source_w: source.0,
        source_h: source.1,
        renderer: "gpu".into(),
        is_raw,
    })
}
