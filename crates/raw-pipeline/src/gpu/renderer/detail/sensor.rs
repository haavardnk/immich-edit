use std::sync::Arc;

use wgpu::{BindGroupLayout, CommandEncoderDescriptor, ComputePipeline};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::geom::dng_warp::DngWarp;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::dng_warp::DngWarpParams;
use crate::gpu::passes::sensor::SensorParams;
use crate::gpu::renderer::{CachedFrame, GpuRenderer};
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view, texture_2d};
use crate::gpu::uniform_pool::PooledUniform;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_sensor(
        &self,
        src: &Arc<CachedFrame>,
        edits: &Edits,
    ) -> PipelineResult<Arc<CachedFrame>> {
        let _span =
            tracing::debug_span!("gpu.submit_sensor", w = src.width, h = src.height).entered();
        let params = SensorParams::from_edits(&edits.lens, src.width, src.height);
        let pass = &self.passes.sensor_stage.sensor;
        Ok(self.submit_resample_pass(
            src,
            &pass.layout,
            &pass.pipeline,
            self.uniform(&params, "sensor-uniform"),
            "sensor",
        ))
    }

    pub(in crate::gpu::renderer) fn submit_dng_warp(
        &self,
        src: &Arc<CachedFrame>,
        warp: &DngWarp,
    ) -> PipelineResult<Arc<CachedFrame>> {
        let _span =
            tracing::debug_span!("gpu.submit_dng_warp", w = src.width, h = src.height).entered();
        let params = DngWarpParams::new(warp, src.width, src.height);
        let pass = &self.passes.sensor_stage.dng_warp;
        Ok(self.submit_resample_pass(
            src,
            &pass.layout,
            &pass.pipeline,
            self.uniform(&params, "dng-warp-uniform"),
            "dng-warp",
        ))
    }

    fn submit_resample_pass(
        &self,
        src: &Arc<CachedFrame>,
        layout: &BindGroupLayout,
        pipeline: &ComputePipeline,
        uniform: PooledUniform,
        label: &str,
    ) -> Arc<CachedFrame> {
        let device = &self.ctx.device;
        let w = src.width;
        let h = src.height;
        let dst = texture_2d(
            device,
            label,
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED,
        );
        let src_view = mip_view(&src.texture, 0);
        let dst_view = mip_view(&dst, 0);
        let bind = bind_group(
            device,
            label,
            layout,
            &[uniform.as_entire_binding(), tex(&src_view), tex(&dst_view)],
        );
        let mut encoder =
            device.create_command_encoder(&CommandEncoderDescriptor { label: Some(label) });
        dispatch_2d(
            &mut encoder,
            label,
            pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &dst, w, h);
        self.ctx.queue.submit(Some(encoder.finish()));
        Arc::new(CachedFrame {
            texture: Arc::new(dst),
            width: w,
            height: h,
            block: src.block,
        })
    }
}
