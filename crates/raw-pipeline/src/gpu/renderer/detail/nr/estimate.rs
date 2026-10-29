use std::sync::atomic::Ordering;

use wgpu::{Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Texture};

use super::profile::{HIST_BLOCKS, NoiseProfile};
use super::sweep::NrSweep;
use super::{chroma, luma};
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::gpu::readback::{map_buffer_cancellable, mapped_range};
use crate::gpu::renderer::GpuRenderer;
use crate::ops::denoise::estimate::HIST_LEN;
use crate::tile::SensorTile;

const HIST_BYTES: u64 = (HIST_BLOCKS * HIST_LEN * size_of::<u32>()) as u64;

pub(in crate::gpu::renderer) struct NoiseHistogram<'r> {
    renderer: &'r GpuRenderer,
    hist: Buffer,
}

impl<'r> NoiseHistogram<'r> {
    pub fn new(renderer: &'r GpuRenderer) -> Self {
        let hist = renderer.ctx.device.create_buffer(&BufferDescriptor {
            label: Some("nr-hist"),
            size: HIST_BYTES,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        Self { renderer, hist }
    }

    pub fn add(&self, src: &Texture, tile: &SensorTile) {
        let rect = tile.detail();
        let (dims, core) = (rect.dims(), tile.hist.within(rect));
        let p = &self.renderer.passes.sensor_stage.nr;
        let mut sweep = NrSweep::new(self.renderer, "nr-estimate");
        luma::estimate(&mut sweep, p, src, dims, core, &self.hist);
        chroma::estimate(&mut sweep, p, src, dims, core, &self.hist);
        sweep.submit();
    }

    pub fn read(self, cancel: Option<&CancelToken>) -> PipelineResult<NoiseProfile> {
        let ctx = &self.renderer.ctx;
        let readback = ctx.device.create_buffer(&BufferDescriptor {
            label: Some("nr-hist-readback"),
            size: HIST_BYTES,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("nr-hist-copy"),
            });
        encoder.copy_buffer_to_buffer(&self.hist, 0, &readback, 0, HIST_BYTES);
        ctx.queue.submit(Some(encoder.finish()));
        map_buffer_cancellable(ctx, &readback, cancel)?;
        let slice = readback.slice(..);
        let data = mapped_range(&slice)?;
        let profile = NoiseProfile::from_histograms(bytemuck::cast_slice(&data[..]));
        drop(data);
        readback.unmap();
        Ok(profile)
    }
}

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn noise_profile_for(
        &self,
        key: u64,
        measure: impl FnOnce() -> PipelineResult<NoiseProfile>,
    ) -> PipelineResult<NoiseProfile> {
        if let Some(profile) = self.sensor.noise_profiles.lock().get(&key).copied() {
            tracing::debug!(target: "gpu_cache", "noise profile cache hit");
            return Ok(profile);
        }
        let _span = tracing::debug_span!("gpu.nr_estimate").entered();
        let profile = measure()?;
        self.sensor.noise_estimates.fetch_add(1, Ordering::Relaxed);
        self.sensor.noise_profiles.lock().put(key, profile);
        Ok(profile)
    }
}
