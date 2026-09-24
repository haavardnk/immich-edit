use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BufferUsages, CommandEncoder, Texture, TextureViewDescriptor};

use super::GpuRenderer;
use super::masks::Retained;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::resources::OutputTargets;

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
}
