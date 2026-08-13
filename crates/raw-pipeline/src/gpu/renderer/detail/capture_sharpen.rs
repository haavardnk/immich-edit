use std::sync::Arc;

use wgpu::{
    CommandEncoderDescriptor, ComputePassDescriptor, Extent3d, Texture, TextureDescriptor,
    TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::gpu::dispatch::{bind_group, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::capture_sharpen::{
    CAPTURE_KERNEL_MAX, CAPTURE_SCRATCH_FORMAT, CaptureApplyParams, CaptureBlurParams,
    CaptureLumaParams,
};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::texture_pool::TextureKey;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn run_capture_sharpen(
        &self,
        src: &Texture,
        dims: (u32, u32),
        sigma: f32,
        key: u64,
    ) -> PipelineResult<Arc<Texture>> {
        if let Some(t) = self.capture_cache.lock().get(&key).cloned() {
            tracing::debug!(target: "gpu_cache", "capture_sharpen cache hit");
            return Ok(t);
        }
        let _span =
            tracing::debug_span!("gpu.run_capture_sharpen", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let (w, h) = dims;
        let p = &self.passes.capture_sharpen;
        let kernel = crate::ops::capture_sharpen::gaussian_kernel(sigma);
        let radius = (kernel.len() / 2) as u32;

        let luma_params = CaptureLumaParams {
            size: [w, h],
            _pad: [0; 2],
        };
        let luma_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&luma_params),
            "capture-luma-u",
        );

        let make_blur_u = |axis: u32, mode: u32, label: &'static str| {
            let mut params = CaptureBlurParams {
                size: [w, h],
                radius,
                axis,
                mode,
                _pad: [0; 3],
                kernel: [0.0; CAPTURE_KERNEL_MAX],
            };
            params.kernel[..kernel.len()].copy_from_slice(&kernel);
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), label)
        };
        let blur_h_buf = make_blur_u(0, 0, "capture-blur-h-u");
        let blur_ratio_buf = make_blur_u(1, 1, "capture-blur-ratio-u");
        let blur_mul_buf = make_blur_u(1, 2, "capture-blur-mul-u");

        let apply_params = CaptureApplyParams {
            size: [w, h],
            radius,
            _pad: 0,
        };
        let apply_buf = self.uniform_pool.acquire(
            device,
            queue,
            bytemuck::bytes_of(&apply_params),
            "capture-apply-u",
        );

        let scratch_key = TextureKey::new(
            CAPTURE_SCRATCH_FORMAT,
            w,
            h,
            1,
            TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
        );
        let luma = self
            .texture_pool
            .acquire(device, scratch_key, "capture-luma");
        let est_a = self
            .texture_pool
            .acquire(device, scratch_key, "capture-est-a");
        let est_b = self
            .texture_pool
            .acquire(device, scratch_key, "capture-est-b");
        let tmp = self
            .texture_pool
            .acquire(device, scratch_key, "capture-tmp");

        let out = device.create_texture(&TextureDescriptor {
            label: Some("capture-sharpen-out"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let src_view = src.create_view(&TextureViewDescriptor::default());
        let luma_view = luma.create_view(&TextureViewDescriptor::default());
        let est_a_view = est_a.create_view(&TextureViewDescriptor::default());
        let est_b_view = est_b.create_view(&TextureViewDescriptor::default());
        let tmp_view = tmp.create_view(&TextureViewDescriptor::default());
        let out_view = out.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });

        let bg_luma = bind_group(
            device,
            "capture-luma-bg",
            &p.luma_layout,
            &[
                luma_buf.as_entire_binding(),
                tex(&src_view),
                tex(&luma_view),
                tex(&est_a_view),
            ],
        );
        let make_blur_bg = |uniform: &crate::gpu::uniform_pool::PooledUniform,
                            read: &wgpu::TextureView,
                            aux: &wgpu::TextureView,
                            write: &wgpu::TextureView| {
            bind_group(
                device,
                "capture-blur-bg",
                &p.blur_layout,
                &[uniform.as_entire_binding(), tex(read), tex(aux), tex(write)],
            )
        };
        let steps: Vec<[wgpu::BindGroup; 4]> =
            [(&est_a_view, &est_b_view), (&est_b_view, &est_a_view)]
                .iter()
                .map(|(src_est, dst_est)| {
                    [
                        make_blur_bg(&blur_h_buf, src_est, &luma_view, &tmp_view),
                        make_blur_bg(&blur_ratio_buf, &tmp_view, &luma_view, dst_est),
                        make_blur_bg(&blur_h_buf, dst_est, &luma_view, &tmp_view),
                        make_blur_bg(&blur_mul_buf, &tmp_view, src_est, dst_est),
                    ]
                })
                .collect();
        let bg_apply = bind_group(
            device,
            "capture-apply-bg",
            &p.apply_layout,
            &[
                apply_buf.as_entire_binding(),
                tex(&src_view),
                tex(&luma_view),
                tex(&est_a_view),
                tex(&out_view),
            ],
        );

        let gx = w.div_ceil(16);
        let gy = h.div_ceil(16);
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("capture-sharpen-enc"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("capture-sharpen-pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&p.luma_pipeline);
            cpass.set_bind_group(0, &bg_luma, &[]);
            cpass.dispatch_workgroups(gx, gy, 1);
            cpass.set_pipeline(&p.blur_pipeline);
            for i in 0..crate::ops::capture_sharpen::ITERATIONS {
                for bg in steps[i % 2].iter() {
                    cpass.set_bind_group(0, bg, &[]);
                    cpass.dispatch_workgroups(gx, gy, 1);
                }
            }
            cpass.set_pipeline(&p.apply_pipeline);
            cpass.set_bind_group(0, &bg_apply, &[]);
            cpass.dispatch_workgroups(gx, gy, 1);
        }
        self.encode_mipgen(&mut encoder, &out, w, h);
        queue.submit(Some(encoder.finish()));

        let out = Arc::new(out);
        self.capture_cache.lock().put(key, out.clone());
        Ok(out)
    }
}
