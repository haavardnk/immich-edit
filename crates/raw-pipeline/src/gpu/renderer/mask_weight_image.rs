use wgpu::BufferUsages;
use wgpu::CommandEncoder;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use super::GpuRenderer;
use super::masks::Retained;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::resources::OutputTargets;
use crate::gpu::texture::full_view;

impl GpuRenderer {
    pub(super) fn encode_mask_weight_image(
        &self,
        encoder: &mut CommandEncoder,
        p: &OutputTargets,
        out_dims: (u32, u32),
        retained: &mut Retained,
    ) {
        let device = &self.ctx.device;
        let params = crate::gpu::passes::mask_weight_image::pack_params(out_dims.0, out_dims.1);
        let params_buf = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("mask-weight-image-uniform"),
            contents: bytemuck::bytes_of(&params),
            usage: BufferUsages::UNIFORM,
        });
        let weight_view = full_view(&p.mask_weight);
        let dst_view = full_view(&p.mask_scratch_tone);
        let bind = bind_group(
            device,
            "mask-weight-image-bg",
            &self.passes.mask_weight_image.layout,
            &[
                params_buf.as_entire_binding(),
                tex(&weight_view),
                tex(&dst_view),
            ],
        );
        dispatch_2d(
            encoder,
            "mask-weight-image",
            &self.passes.mask_weight_image.pipeline,
            &bind,
            out_dims.0.div_ceil(16),
            out_dims.1.div_ceil(16),
        );
        retained.bufs.push(params_buf);
        retained.binds.push(bind);
    }
}
