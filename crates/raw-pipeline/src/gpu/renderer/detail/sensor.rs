use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::Ordering;

use wgpu::{BindGroupLayout, CommandEncoderDescriptor, ComputePipeline, TextureUsages};

use crate::edits::LensEdits;
use crate::frame::RawFrame;
use crate::geom::dng_warp::DngWarp;
use crate::gpu::dispatch::{bind_group, dispatch_2d, tex};
use crate::gpu::helpers::mip_count;
use crate::gpu::passes::dng_warp::DngWarpParams;
use crate::gpu::passes::sensor::SensorParams;
use crate::gpu::renderer::stage_cache::Stage;
use crate::gpu::renderer::{GpuRenderer, SensorFrame};
use crate::gpu::texture::{STORAGE_SAMPLED, mip_view, texture_2d};
use crate::gpu::uniform_pool::PooledUniform;
use crate::tile::{Rect, Warp};

fn hash_rect(rect: Rect, h: &mut impl Hasher) {
    (rect.x, rect.y, rect.w, rect.h).hash(h);
}

fn hash_warp(warp: &Warp, h: &mut impl Hasher) {
    match warp {
        Warp::Lens(lens) => {
            0u8.hash(h);
            lens.hash_key(h);
        }
        Warp::Dng(dng) => {
            1u8.hash(h);
            let planes = dng
                .planes
                .iter()
                .flat_map(|p| p.radial.iter().chain(&p.tangential));
            for v in dng.center.iter().chain(planes) {
                v.to_bits().hash(h);
            }
        }
    }
}

fn warp_key(input: u64, warp: &Warp, src: Rect, dst: Rect, frame: (u32, u32)) -> u64 {
    let mut h = DefaultHasher::new();
    input.hash(&mut h);
    hash_warp(warp, &mut h);
    hash_rect(src, &mut h);
    hash_rect(dst, &mut h);
    frame.hash(&mut h);
    h.finish()
}

impl GpuRenderer {
    pub fn warp_passes(&self) -> u64 {
        self.sensor.warps.load(Ordering::Relaxed)
    }

    pub(in crate::gpu::renderer) fn base_identity(frame: &RawFrame, src: &SensorFrame) -> u64 {
        let mut h = DefaultHasher::new();
        frame.id.hash(&mut h);
        src.block.hash(&mut h);
        hash_rect(src.rect, &mut h);
        h.finish()
    }

    pub(in crate::gpu::renderer) fn sensor_warp(
        &self,
        src: &Arc<SensorFrame>,
        warp: &Warp,
        dst: Rect,
        frame: (u32, u32),
        input: Option<u64>,
    ) -> (Arc<SensorFrame>, Option<u64>) {
        let key = input.map(|input| warp_key(input, warp, src.rect, dst, frame));
        if let Some(texture) = key.and_then(|key| self.sensor.stages.get(Stage::Warp, key)) {
            tracing::debug!(target: "gpu_cache", "warp cache hit");
            let warped = Arc::new(SensorFrame {
                texture,
                rect: dst,
                block: src.block,
            });
            return (warped, key);
        }
        let warped = match warp {
            Warp::Dng(dng) => self.submit_dng_warp(src, dng, dst, frame),
            Warp::Lens(lens) => self.submit_sensor(src, lens, dst, frame),
        };
        self.sensor.warps.fetch_add(1, Ordering::Relaxed);
        if let Some(key) = key {
            self.sensor
                .stages
                .put(Stage::Warp, key, warped.texture.clone());
        }
        (warped, key)
    }

    fn submit_sensor(
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

    fn submit_dng_warp(
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
