use wgpu::Texture;

use super::GpuRenderer;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::texture::mip_view;

impl GpuRenderer {
    pub(super) fn encode_mipgen(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &Texture,
        w: u32,
        h: u32,
    ) {
        let _span = tracing::trace_span!("gpu.mipgen", w = w, h = h).entered();
        let levels = mip_count(w, h);
        if levels <= 1 {
            return;
        }
        let device = &self.ctx.device;
        let mut mip_w = w;
        let mut mip_h = h;
        for level in 1..levels {
            let src_view = mip_view(texture, level - 1);
            let dst_w = (mip_w / 2).max(1);
            let dst_h = (mip_h / 2).max(1);
            let dst_view = mip_view(texture, level);
            let bind = bind_group(
                device,
                "mipgen-bg",
                &self.passes.mipgen.layout,
                &[tex(&src_view), tex(&dst_view)],
            );
            dispatch_2d(
                encoder,
                "mipgen-pass",
                &self.passes.mipgen.pipeline,
                &bind,
                dst_w.div_ceil(16),
                dst_h.div_ceil(16),
            );
            mip_w = dst_w;
            mip_h = dst_h;
        }
    }
}
