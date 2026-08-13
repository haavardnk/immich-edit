use std::sync::Arc;

use wgpu::{
    CommandEncoderDescriptor, Extent3d, TextureDescriptor, TextureDimension, TextureUsages,
    TextureViewDescriptor,
};

use crate::PipelineResult;
use crate::edits::Edits;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::sensor::SensorParams;
use crate::gpu::renderer::{CachedFrame, GpuRenderer};

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn run_sensor(
        &self,
        src: &Arc<CachedFrame>,
        edits: &Edits,
    ) -> PipelineResult<Arc<CachedFrame>> {
        let _span = tracing::debug_span!("gpu.run_sensor", w = src.width, h = src.height).entered();
        let device = &self.ctx.device;
        let queue = &self.ctx.queue;
        let w = src.width;
        let h = src.height;
        let params = SensorParams::from_edits(&edits.lens, w, h);
        let uniform_buf =
            self.uniform_pool
                .acquire(device, queue, bytemuck::bytes_of(&params), "sensor-uniform");
        let dst = device.create_texture(&TextureDescriptor {
            label: Some("sensor-out"),
            size: Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count(w, h),
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.ctx.linear_format,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let src_view = src.texture.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let dst_view = dst.create_view(&TextureViewDescriptor {
            base_mip_level: 0,
            mip_level_count: Some(1),
            ..Default::default()
        });
        let pass = &self.passes.sensor;
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
        }))
    }
}
