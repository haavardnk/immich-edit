use std::sync::Arc;

use wgpu::Texture;

use super::{DetailJob, SensorFrame, SensorJob};
use crate::PipelineResult;
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::detail::{NoiseHistogram, NoiseProfile};
use crate::gpu::renderer::resample::Resample;
use crate::gpu::source::LinearSource;
use crate::tile::{Rect, SensorTile, Warp};
use crate::timing;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn sensor_window(
        &self,
        job: &SensorJob<'_>,
        window: Rect,
        profile: Option<NoiseProfile>,
    ) -> PipelineResult<LinearSource> {
        let tile = job.shape.chain.tile(window);
        let (texture, extent) = self.sensor_tile(job, &tile, profile)?;
        Ok(job.window_source(texture, extent))
    }

    pub(super) fn sensor_tile(
        &self,
        job: &SensorJob<'_>,
        tile: &SensorTile,
        profile: Option<NoiseProfile>,
    ) -> PipelineResult<(Arc<Texture>, Rect)> {
        let base = self.sensor_base(job, tile)?;
        match &job.detail {
            Some(detail) => {
                let wb = self.sensor_wb(job, detail, &base)?;
                self.sensor_detail(job, detail, tile, &wb, profile)
            }
            None => Ok((base.texture.clone(), base.rect)),
        }
    }

    pub(super) fn sensor_base(
        &self,
        job: &SensorJob<'_>,
        tile: &SensorTile,
    ) -> PipelineResult<Arc<SensorFrame>> {
        let (frame, shape, t) = (job.frame, &job.shape, job.t);
        let mut cached = t.stage(timing::DEMOSAIC, || {
            if job.cached() {
                self.get_or_demosaic(frame, shape)
            } else {
                self.demosaic(frame, shape, tile.input())
            }
        })?;
        job.check()?;
        if let Some((plane, mix)) = job.denoise {
            cached = t.stage(timing::AI_DENOISE, || {
                self.submit_denoise_blend(frame, &cached, plane, mix, job.cached())
            })?;
            job.check()?;
        }
        for (warp, &dst) in shape.chain.warps.iter().zip(&tile.rects[1..]) {
            cached = t.stage(timing::LENS, || match warp {
                Warp::Dng(dng) => self.submit_dng_warp(&cached, dng, dst, shape.sensor()),
                Warp::Lens(lens) => self.submit_sensor(&cached, lens, dst, shape.sensor()),
            });
            job.check()?;
        }
        Ok(cached)
    }

    pub(super) fn sensor_wb(
        &self,
        job: &SensorJob<'_>,
        detail: &DetailJob,
        base: &SensorFrame,
    ) -> PipelineResult<Arc<Texture>> {
        let (frame, edits) = (job.frame, &job.edits);
        let wb = job.t.stage(timing::WB_PREPARE, || {
            let key = job.cache(detail.keys.wb);
            self.submit_wb_prepare(base, &frame.meta, edits, &detail.setup, key)
        })?;
        job.check()?;
        if !edits.retouch_active() {
            return Ok(wb);
        }
        let retouched = job.t.stage(timing::RETOUCH, || {
            self.submit_retouch(wb, base.rect.dims(), frame, edits)
        })?;
        job.check()?;
        Ok(retouched)
    }

    pub(super) fn sensor_detail(
        &self,
        job: &SensorJob<'_>,
        detail: &DetailJob,
        tile: &SensorTile,
        wb: &Arc<Texture>,
        profile: Option<NoiseProfile>,
    ) -> PipelineResult<(Arc<Texture>, Rect)> {
        let (shape, edits, t) = (&job.shape, &job.edits, job.t);
        let rect = tile.detail();
        let src = if job.nr_active() {
            let measure = || match profile {
                Some(profile) => Ok(profile),
                None => self.noise_profile_for(detail.keys.nr_source, || {
                    let histogram = NoiseHistogram::new(self);
                    histogram.add(wb, tile);
                    histogram.read(job.cancel)
                }),
            };
            let key = job.cache(detail.keys.nr);
            let out = t.stage(timing::NOISE_REDUCTION, || {
                self.submit_nr(wb, rect.dims(), edits, key, measure)
            })?;
            job.check()?;
            out
        } else {
            wb.clone()
        };
        let src = match shape.sigma {
            Some(sigma) => {
                let key = job.cache(detail.keys.capture(sigma));
                let out = t.stage(timing::CAPTURE_SHARPEN, || {
                    self.submit_capture_sharpen(&src, rect, shape.sensor(), sigma, key)
                })?;
                job.check()?;
                out
            }
            None => src,
        };
        if !shape.resamples() {
            return Ok((src, rect));
        }
        let geometry = Resample {
            src: rect,
            dst: tile.core,
            src_frame: shape.sensor(),
            dst_frame: shape.working(),
        };
        let mips = job.cached() && job.plan.atmosphere;
        let out = t.stage(timing::PREVIEW_RESAMPLE, || {
            self.resample_lanczos(&src, geometry, "preview-spatial-src", mips)
        })?;
        job.check()?;
        Ok((out, tile.core))
    }
}
