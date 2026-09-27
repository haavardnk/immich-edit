use std::sync::Arc;

use wgpu::{CommandEncoderDescriptor, Texture, TextureUsages};

use crate::PipelineResult;
use crate::gpu::dispatch::{begin_pass, bind_group, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::capture_sharpen::{
    CAPTURE_KERNEL_MAX, CAPTURE_SCRATCH_FORMAT, CaptureApplyParams, CaptureBlurParams,
    CaptureLumaParams,
};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::stage_cache::Stage;
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view, texture_2d};
use crate::gpu::texture_pool::TextureKey;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_capture_sharpen(
        &self,
        src: &Texture,
        dims: (u32, u32),
        sigma: f32,
        key: u64,
    ) -> PipelineResult<Arc<Texture>> {
        if let Some(t) = self.sensor.stages.get(Stage::Capture, key) {
            tracing::debug!(target: "gpu_cache", "capture_sharpen cache hit");
            return Ok(t);
        }
        let _span =
            tracing::debug_span!("gpu.submit_capture_sharpen", w = dims.0, h = dims.1).entered();
        let device = &self.ctx.device;
        let (w, h) = dims;
        let p = &self.passes.sensor_stage.capture_sharpen;
        let kernel = crate::ops::capture_sharpen::gaussian_kernel(sigma);
        let radius = (kernel.len() / 2) as i32;

        let luma_buf = self.uniform(
            &CaptureLumaParams {
                size: [w, h],
                _pad: [0; 2],
            },
            "capture-luma-u",
        );

        let make_blur_u = |axis: i32, mode: u32, label: &'static str| {
            let mut params = CaptureBlurParams {
                size: [w, h],
                radius,
                axis,
                mode,
                _pad: [0; 3],
                kernel: [0.0; CAPTURE_KERNEL_MAX],
            };
            params.kernel[..kernel.len()].copy_from_slice(&kernel);
            self.uniform(&params, label)
        };
        let blur_h_buf = make_blur_u(0, 0, "capture-blur-h-u");
        let blur_ratio_buf = make_blur_u(1, 1, "capture-blur-ratio-u");
        let blur_mul_buf = make_blur_u(1, 2, "capture-blur-mul-u");

        let apply_buf = self.uniform(
            &CaptureApplyParams {
                size: [w, h],
                radius,
                _pad: 0,
            },
            "capture-apply-u",
        );

        let scratch_key = TextureKey::new(CAPTURE_SCRATCH_FORMAT, w, h, 1, STORAGE_SAMPLED);
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

        let out = texture_2d(
            device,
            "capture-sharpen-out",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_SRC,
        );

        let src_view = full_view(src);
        let luma_view = full_view(&luma);
        let est_a_view = full_view(&est_a);
        let est_b_view = full_view(&est_b);
        let tmp_view = full_view(&tmp);
        let out_view = mip_view(&out, 0);

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
            let mut cpass = begin_pass(&mut encoder, "capture-sharpen-pass");
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
        self.ctx.queue.submit(Some(encoder.finish()));

        let out = Arc::new(out);
        self.sensor.stages.put(Stage::Capture, key, out.clone());
        Ok(out)
    }
}
