use std::sync::Arc;

use wgpu::{BindGroupLayout, CommandEncoderDescriptor, ComputePipeline, TextureUsages};

use crate::edits::LensEdits;
use crate::geom::dng_warp::DngWarp;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::dng_warp::DngWarpParams;
use crate::gpu::passes::sensor::SensorParams;
use crate::gpu::renderer::{GpuRenderer, SensorFrame};
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view, texture_2d};
use crate::gpu::uniform_pool::PooledUniform;
use crate::tile::Rect;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_sensor(
        &self,
        src: &Arc<SensorFrame>,
        lens: &LensEdits,
        dst: Rect,
        frame: (u32, u32),
    ) -> Arc<SensorFrame> {
        let _span = tracing::debug_span!("gpu.submit_sensor", w = dst.w, h = dst.h).entered();
        let params = SensorParams::new(lens, frame, src.rect, dst);
        let pass = &self.passes.sensor_stage.sensor;
        self.submit_warp_pass(
            src,
            dst,
            &pass.layout,
            &pass.pipeline,
            self.uniform(&params, "sensor-uniform"),
            "sensor",
        )
    }

    pub(in crate::gpu::renderer) fn submit_dng_warp(
        &self,
        src: &Arc<SensorFrame>,
        warp: &DngWarp,
        dst: Rect,
        frame: (u32, u32),
    ) -> Arc<SensorFrame> {
        let _span = tracing::debug_span!("gpu.submit_dng_warp", w = dst.w, h = dst.h).entered();
        let params = DngWarpParams::new(warp, frame, src.rect, dst);
        let pass = &self.passes.sensor_stage.dng_warp;
        self.submit_warp_pass(
            src,
            dst,
            &pass.layout,
            &pass.pipeline,
            self.uniform(&params, "dng-warp-uniform"),
            "dng-warp",
        )
    }

    fn submit_warp_pass(
        &self,
        src: &Arc<SensorFrame>,
        rect: Rect,
        layout: &BindGroupLayout,
        pipeline: &ComputePipeline,
        uniform: PooledUniform,
        label: &str,
    ) -> Arc<SensorFrame> {
        let device = &self.ctx.device;
        let (w, h) = rect.dims();
        let dst = texture_2d(
            device,
            label,
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED | TextureUsages::COPY_SRC,
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
        Arc::new(SensorFrame {
            texture: Arc::new(dst),
            rect,
            block: src.block,
        })
    }
}
