use std::sync::Arc;

use wgpu::CommandEncoderDescriptor;

use crate::PipelineResult;
use crate::edits::Edits;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::sensor::SensorParams;
use crate::gpu::renderer::{CachedFrame, GpuRenderer};
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view, texture_2d};

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn submit_sensor(
        &self,
        src: &Arc<CachedFrame>,
        edits: &Edits,
    ) -> PipelineResult<Arc<CachedFrame>> {
        let _span =
            tracing::debug_span!("gpu.submit_sensor", w = src.width, h = src.height).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = src.width;
        let h = src.height;
        let uniform_buf = self.uniform(
            &SensorParams::from_edits(&edits.lens, w, h),
            "sensor-uniform",
        );
        let dst = texture_2d(
            device,
            "sensor-out",
            self.ctx.linear_format,
            (w, h),
            mip_count(w, h),
            STORAGE_SAMPLED,
        );
        let src_view = mip_view(&src.texture, 0);
        let dst_view = mip_view(&dst, 0);
        let pass = &self.passes.sensor_stage.sensor;
        let bind = bind_group(
            device,
            "sensor-bg",
            &pass.layout,
            &[
                uniform_buf.as_entire_binding(),
                tex(&src_view),
                tex(&dst_view),
            ],
        );
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("sensor-enc"),
        });
        dispatch_2d(
            &mut encoder,
            "sensor-pass",
            &pass.pipeline,
            &bind,
            w.div_ceil(16),
            h.div_ceil(16),
        );
        self.encode_mipgen(&mut encoder, &dst, w, h);
        queue.submit(Some(encoder.finish()));
        Ok(Arc::new(CachedFrame {
            texture: Arc::new(dst),
            width: w,
            height: h,
            block: src.block,
        }))
    }
}
