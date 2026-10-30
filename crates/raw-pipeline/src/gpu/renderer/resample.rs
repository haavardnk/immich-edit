use std::sync::Arc;

use wgpu::{CommandEncoderDescriptor, Texture, TextureUsages};

use crate::PipelineResult;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::resample;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view, texture_2d};
use crate::tile::Rect;

use super::GpuRenderer;

#[derive(Clone, Copy)]
pub(super) struct Resample {
    pub src: Rect,
    pub dst: Rect,
    pub src_frame: (u32, u32),
    pub dst_frame: (u32, u32),
}

impl GpuRenderer {
    pub(super) fn resample_lanczos(
        &self,
        src: &Texture,
        geometry: Resample,
        label: &str,
        mips: bool,
    ) -> PipelineResult<Arc<Texture>> {
        let Resample {
            src: src_rect,
            dst: dst_rect,
            src_frame,
            dst_frame,
        } = geometry;
        let (sw, sh) = src_rect.dims();
        let (dw, dh) = dst_rect.dims();
        let _span = tracing::debug_span!("gpu.resample_lanczos", sw, sh, dw, dh).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;

        let make_texture = |w: u32, h: u32, label: &str, levels: u32| {
            texture_2d(
                device,
                label,
                self.ctx.linear_format,
                (w, h),
                levels,
                STORAGE_SAMPLED | TextureUsages::COPY_SRC,
            )
        };

        let tmp = make_texture(dw, sh, "resample-tmp", 1);
        let levels = if mips { mip_count(dw, dh) } else { 1 };
        let dst = Arc::new(make_texture(dw, dh, label, levels));

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("resample-enc"),
        });
        let pass = &self.passes.resample;
        let steps = [
            (
                (dw, sh),
                (sw, sh),
                src_frame.0 as f32 / dst_frame.0 as f32,
                0u32,
                (dst_rect.x, src_rect.x),
                src,
                &tmp,
            ),
            (
                (dw, dh),
                (dw, sh),
                src_frame.1 as f32 / dst_frame.1 as f32,
                1u32,
                (dst_rect.y, src_rect.y),
                &tmp,
                &*dst,
            ),
        ];
        let mut binds = Vec::with_capacity(steps.len());
        let mut uniforms = Vec::with_capacity(steps.len());
        for (out_dims, in_dims, scale, axis, origins, input, output) in steps {
            let uniform = self.uniform(
                &resample::pack_params(out_dims, in_dims, scale, axis, origins),
                "resample-uniform",
            );
            let in_view = full_view(input);
            let out_view = mip_view(output, 0);
            let bind = bind_group(
                device,
                "resample-bg",
                &pass.layout,
                &[uniform.as_entire_binding(), tex(&in_view), tex(&out_view)],
            );
            binds.push((bind, out_dims));
            uniforms.push(uniform);
        }
        for (bind, (w, h)) in &binds {
            dispatch_2d(
                &mut encoder,
                "resample-pass",
                &pass.pipeline,
                bind,
                w.div_ceil(16),
                h.div_ceil(16),
            );
        }
        if mips {
            self.encode_mipgen(&mut encoder, &dst, dw, dh);
        }
        queue.submit(Some(encoder.finish()));
        Ok(dst)
    }
}
