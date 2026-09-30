use std::sync::Arc;

use wgpu::{CommandEncoderDescriptor, Texture, TextureUsages};

use crate::PipelineResult;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::resample;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view, texture_2d};

use super::GpuRenderer;

impl GpuRenderer {
    pub(super) fn resample_lanczos(
        &self,
        src: &Texture,
        src_dims: (u32, u32),
        dst_dims: (u32, u32),
        label: &str,
        mips: bool,
    ) -> PipelineResult<Arc<Texture>> {
        let _span = tracing::debug_span!(
            "gpu.resample_lanczos",
            sw = src_dims.0,
            sh = src_dims.1,
            dw = dst_dims.0,
            dh = dst_dims.1
        )
        .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (sw, sh) = src_dims;
        let (dw, dh) = dst_dims;

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
            ((dw, sh), (sw, sh), sw as f32 / dw as f32, 0u32, src, &tmp),
            ((dw, dh), (dw, sh), sh as f32 / dh as f32, 1u32, &tmp, &*dst),
        ];
        let mut binds = Vec::with_capacity(steps.len());
        let mut uniforms = Vec::with_capacity(steps.len());
        for (out_dims, in_dims, scale, axis, input, output) in steps {
            let uniform = self.uniform(
                &resample::pack_params(out_dims, in_dims, scale, axis),
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
