use std::sync::Arc;

use wgpu::{CommandEncoderDescriptor, PollType, TextureUsages};

use super::{DetailJob, SensorJob};
use crate::gpu::dispatch::{Region, submit_copy};
use crate::gpu::helpers::mip_count;
use crate::gpu::limits::check_texture;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::detail::{NoiseHistogram, NoiseProfile};
use crate::gpu::source::LinearSource;
use crate::gpu::texture::{STORAGE_SAMPLED, texture_2d};
use crate::tile::SensorTile;
use crate::{PipelineError, PipelineResult};

impl GpuRenderer {
    pub(super) fn sensor_tiled(
        &self,
        job: &SensorJob<'_>,
        edge: u32,
    ) -> PipelineResult<LinearSource> {
        let tiles = job.shape.chain.tiles(edge);
        let (w, h) = job.shape.working();
        check_texture(&self.ctx.device.limits(), "working image", (w, h))?;
        let levels = if job.working_mips() {
            mip_count(w, h)
        } else {
            1
        };
        let working = Arc::new(texture_2d(
            &self.ctx.device,
            "sensor-tiles",
            self.ctx.linear_format,
            (w, h),
            levels,
            STORAGE_SAMPLED | TextureUsages::COPY_DST | TextureUsages::COPY_SRC,
        ));
        let pooled = self.texture_pool.bytes();
        let profile = self.tiled_noise_profile(job, &tiles)?;
        for tile in &tiles {
            let (texture, extent) = self.sensor_tile(job, tile, profile)?;
            let src = tile.core.within(extent);
            submit_copy(
                &self.ctx,
                "sensor-tile-place",
                Region::at(&texture, 0, (src.x, src.y)),
                Region::at(&working, 0, (tile.core.x, tile.core.y)),
                tile.core.dims(),
            );
            self.wait_idle(pooled)?;
            job.check()?;
        }
        tracing::debug!(tiles = tiles.len(), edge, w, h, "gpu sensor tiles");
        if job.working_mips() {
            let mut encoder = self
                .ctx
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("sensor-tiles-mipgen"),
                });
            self.encode_mipgen(&mut encoder, &working, w, h);
            self.ctx.queue.submit(Some(encoder.finish()));
        }
        let atmosphere = self.atmosphere(job, &working)?;
        Ok(job.source(working, atmosphere))
    }

    pub(in crate::gpu::renderer) fn tiled_noise_profile(
        &self,
        job: &SensorJob<'_>,
        tiles: &[SensorTile],
    ) -> PipelineResult<Option<NoiseProfile>> {
        match &job.detail {
            Some(detail) if job.nr_active() => self
                .noise_profile_for(detail.keys.nr_source, || {
                    self.measure_tiled_noise(job, detail, tiles)
                })
                .map(Some),
            _ => Ok(None),
        }
    }

    fn measure_tiled_noise(
        &self,
        job: &SensorJob<'_>,
        detail: &DetailJob,
        tiles: &[SensorTile],
    ) -> PipelineResult<NoiseProfile> {
        let pooled = self.texture_pool.bytes();
        let histogram = NoiseHistogram::new(self);
        for tile in tiles {
            let base = self.sensor_base(job, tile)?;
            let wb = self.sensor_wb(job, detail, &base)?;
            histogram.add(&wb, tile);
            self.wait_idle(pooled)?;
            job.check()?;
        }
        histogram.read(job.cancel)
    }

    pub(in crate::gpu::renderer) fn wait_idle(&self, pooled: u64) -> PipelineResult<()> {
        self.ctx
            .device
            .poll(PollType::wait_indefinitely())
            .map_err(|e| PipelineError::Render(format!("gpu tile wait: {e}")))?;
        self.texture_pool
            .trim(self.texture_pool.bytes().saturating_sub(pooled));
        Ok(())
    }
}
