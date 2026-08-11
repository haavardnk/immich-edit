use std::sync::Arc;

use wgpu::{
    BindGroupDescriptor, BindGroupEntry, CommandEncoderDescriptor, ComputePassDescriptor, Extent3d,
    Texture, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::gpu::passes::resample;

use super::GpuRenderer;

impl GpuRenderer {
    pub(super) fn resample_lanczos(
        &self,
        src: &Texture,
        src_dims: (u32, u32),
        dst_dims: (u32, u32),
        label: &str,
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

        let make_texture = |w: u32, h: u32, label: &str| {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: self.ctx.linear_format,
                usage: TextureUsages::STORAGE_BINDING
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };

        let tmp = make_texture(dw, sh, "resample-tmp");
        let dst = Arc::new(make_texture(dw, dh, label));

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
            let params = resample::pack_params(out_dims, in_dims, scale, axis);
            let uniform = self
                .uniform_pool
                .acquire(device, queue, &params, "resample-uniform");
            let in_view = input.create_view(&TextureViewDescriptor::default());
            let out_view = output.create_view(&TextureViewDescriptor::default());
            let bind = device.create_bind_group(&BindGroupDescriptor {
                label: Some("resample-bg"),
                layout: &pass.layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&in_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&out_view),
                    },
                ],
            });
            binds.push((bind, out_dims));
            uniforms.push(uniform);
        }
        for (bind, (w, h)) in &binds {
            let mut cp = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("resample-pass"),
                timestamp_writes: None,
            });
            cp.set_pipeline(&pass.pipeline);
            cp.set_bind_group(0, bind, &[]);
            cp.dispatch_workgroups(w.div_ceil(16), h.div_ceil(16), 1);
        }
        queue.submit(Some(encoder.finish()));
        Ok(dst)
    }
}
