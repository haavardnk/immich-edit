use std::sync::Arc;

use wgpu::{CommandEncoderDescriptor, Texture, TextureUsages};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::FrameMeta;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::renderer::stage_cache::Stage;
use crate::gpu::renderer::uniform::build_process_uniform;
use crate::gpu::renderer::{CachedFrame, GpuRenderer};
use crate::gpu::texture::{STORAGE_SAMPLED, full_view, mip_view, texture_2d};
use crate::gpu::uniforms::{FULL_WINDOW, ProcessHeader};
use crate::ops::{OpContext, OpScratch, RenderContext};

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_wb_prepare(
        &self,
        cached: &CachedFrame,
        meta: &FrameMeta,
        edits: &Edits,
        setup: &crate::dcp::setup::DcpSetup,
        key: u64,
    ) -> PipelineResult<Arc<Texture>> {
        if let Some(t) = self.sensor.stages.get(Stage::Wb, key) {
            tracing::debug!(target: "gpu_cache", "wb_base cache hit");
            return Ok(t);
        }
        let _span =
            tracing::debug_span!("gpu.submit_wb_prepare", w = cached.width, h = cached.height)
                .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = cached.width;
        let h = cached.height;

        let ctx_op = OpContext {
            render: RenderContext {
                wb_coeffs: meta.wb_coeffs,
                cam_to_srgb: setup.cam_to_srgb,
                is_raw: meta.is_raw,
                capture_sigma: meta.capture_sigma,
                preview_mode: crate::frame::PreviewMode::None,
                roi: None,
                dcp: setup.resolved.clone(),
                output_scale: 1.0,
            },
            scratch: OpScratch::default(),
        };

        let pass = &self.passes.sensor_stage.wb_prepare;
        let uniform_bytes = build_process_uniform(
            &pass.built,
            &self.passes.registry,
            edits,
            &ctx_op,
            &ProcessHeader {
                src_size: [w, h],
                out_size: [w, h],
                crop: [0.0, 0.0, 1.0, 1.0],
                flags: [0, 0, 0, 0],
                geom_extra: [0.0; 4],
                active_mask: [0; 4],
                geom_extra2: [0.0; 4],
                geom_extra3: [0.0; 4],
                output: [0, 0, 0, 0],
                perspective: crate::geom::perspective::IDENTITY_ROWS,
                src_window: FULL_WINDOW,
            },
        );

        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "wb-prepare-uniform");

        let wb_base = texture_2d(
            device,
            "wb-base",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_SRC,
        );

        let src_view = full_view(&cached.texture);
        let dst_view = mip_view(&wb_base, 0);
        let bind = bind_group(
            device,
            "wb-prepare-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                tex(&dst_view),
            ],
        );

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("wb-prepare-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "wb-prepare-pass",
            &pass.pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &wb_base, w, h);
        queue.submit(Some(encoder.finish()));

        let out = Arc::new(wb_base);
        self.sensor.stages.put(Stage::Wb, key, out.clone());
        Ok(out)
    }
}
