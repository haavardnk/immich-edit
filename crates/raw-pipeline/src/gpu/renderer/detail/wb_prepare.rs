use std::sync::Arc;

use wgpu::{
    CommandEncoderDescriptor, Extent3d, Texture, TextureDescriptor, TextureDimension,
    TextureUsages, TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::frame::RawFrame;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::renderer::cache_keys::wb_cache_key;
use crate::gpu::renderer::uniform::build_process_uniform;
use crate::gpu::renderer::{CachedFrame, GpuRenderer};
use crate::gpu::uniforms::ProcessHeader;
use crate::ops::{OpContext, OpScratch, RenderContext};

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn run_wb_prepare(
        &self,
        cached: &CachedFrame,
        frame: &RawFrame,
        edits: &Edits,
        setup: &crate::dcp_pipeline::DcpSetup,
    ) -> PipelineResult<Arc<Texture>> {
        let key = wb_cache_key(
            frame,
            edits,
            (cached.width, cached.height),
            setup.cam_to_srgb,
        );
        if let Some(t) = self.wb_cache.lock().get(&key).cloned() {
            tracing::debug!(target: "gpu_cache", "wb_base cache hit");
            return Ok(t);
        }
        let _span = tracing::debug_span!("gpu.run_wb_prepare", w = cached.width, h = cached.height)
            .entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = cached.width;
        let h = cached.height;

        let ctx_op = OpContext {
            render: RenderContext {
                wb_coeffs: frame.wb_coeffs,
                cam_to_srgb: setup.cam_to_srgb,
                is_raw: frame.is_raw,
                capture_sigma: frame.capture_sigma,
                preview_mode: crate::frame::PreviewMode::None,
                roi: None,
                dcp: setup.resolved.clone(),
            },
            scratch: OpScratch::default(),
        };

        let pass = &self.passes.wb_prepare;
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
                perspective: crate::perspective::IDENTITY_ROWS,
            },
        );

        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, &uniform_bytes, "wb-prepare-uniform");

        let wb_base = device.create_texture(&TextureDescriptor {
            label: Some("wb-base"),
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

        let src_view = cached
            .texture
            .create_view(&TextureViewDescriptor::default());
        let dst_view = wb_base.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
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
        self.wb_cache.lock().put(key, out.clone());
        Ok(out)
    }
}
