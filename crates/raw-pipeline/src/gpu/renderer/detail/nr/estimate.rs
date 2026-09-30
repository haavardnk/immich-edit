use std::sync::atomic::Ordering;

use wgpu::{Buffer, BufferDescriptor, BufferUsages, Texture};

use super::profile::{HIST_BLOCKS, NoiseProfile};
use super::sweep::NrSweep;
use super::{chroma, luma};
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::gpu::readback::{map_buffer_cancellable, mapped_range};
use crate::gpu::renderer::GpuRenderer;
use crate::ops::denoise::estimate::HIST_LEN;

const HIST_BYTES: u64 = (HIST_BLOCKS * HIST_LEN * size_of::<u32>()) as u64;

impl GpuRenderer {
    pub(super) fn noise_profile_for(
        &self,
        key: u64,
        src: &Texture,
        dims: (u32, u32),
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<NoiseProfile> {
        if let Some(profile) = self.sensor.noise_profiles.lock().get(&key).copied() {
            tracing::debug!(target: "gpu_cache", "noise profile cache hit");
            return Ok(profile);
        }
        let _span = tracing::debug_span!("gpu.nr_estimate", w = dims.0, h = dims.1).entered();
        let profile = self.estimate_noise(src, dims, cancel)?;
        self.sensor.noise_estimates.fetch_add(1, Ordering::Relaxed);
        self.sensor.noise_profiles.lock().put(key, profile);
        Ok(profile)
    }

    fn estimate_noise(
        &self,
        src: &Texture,
        dims: (u32, u32),
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<NoiseProfile> {
        let buffer = |label: &'static str, usage: BufferUsages| -> Buffer {
            self.ctx.device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size: HIST_BYTES,
                usage,
                mapped_at_creation: false,
            })
        };
        let hist = buffer("nr-hist", BufferUsages::STORAGE | BufferUsages::COPY_SRC);
        let readback = buffer(
            "nr-hist-readback",
            BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        );
        let p = &self.passes.sensor_stage.nr;
        let mut sweep = NrSweep::new(self, "nr-estimate");
        luma::estimate(&mut sweep, p, src, dims, &hist);
        chroma::estimate(&mut sweep, p, src, dims, &hist);
        sweep
            .encoder()
            .copy_buffer_to_buffer(&hist, 0, &readback, 0, HIST_BYTES);
        sweep.submit();

        map_buffer_cancellable(&self.ctx, &readback, cancel)?;
        let slice = readback.slice(..);
        let data = mapped_range(&slice)?;
        let profile = NoiseProfile::from_histograms(bytemuck::cast_slice(&data[..]));
        drop(data);
        readback.unmap();
        Ok(profile)
    }
}
