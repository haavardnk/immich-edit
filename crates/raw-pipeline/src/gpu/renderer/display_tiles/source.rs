use std::sync::Arc;
use std::time::Instant;

use super::DisplayTiles;
use super::context::ContextJob;
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::gpu::renderer::GpuRenderer;
use crate::gpu::renderer::sensor_stage::{SensorJob, SourcePlan};
use crate::gpu::timer::RenderTimings;
use crate::source::{ContextPlanes, RenderedSource, WindowRect};
use crate::tile::{Rect, TilePlan, TilingReport};
use crate::timing;

impl GpuRenderer {
    pub(in crate::gpu::renderer) fn render_window_source(
        &self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        tiles: TilePlan,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<RenderedSource> {
        pollster::block_on(self.guarded(async {
            let timings = RenderTimings::new(&self.ctx);
            let plan = SourcePlan::WINDOW;
            let job = SensorJob::new(frame, edits, options, plan, tiles, &timings, cancel)?;
            let edge = tiles.display_edge();
            let layout = DisplayTiles::source(frame, edits, options, job.shape(), edge);
            let window = layout.source_window();
            let windowed = window != Rect::full(job.shape().working());
            let (whole, profile) = match edge {
                Some(edge) => {
                    let partition = job.shape().chain.tiles(edge);
                    (None, self.tiled_noise_profile(&job, &partition)?)
                }
                None => (Some(self.sensor_stage(&job)?), None),
            };
            let planes = match (&layout.context, windowed) {
                (Some(plan), true) => Some(self.source_context(&ContextJob {
                    sensor: &job,
                    options,
                    profile,
                    plan,
                    layout: &layout,
                    whole: whole.as_ref(),
                })?),
                _ => None,
            };
            let linear = match whole {
                Some(linear) => linear,
                None => self.sensor_window(&job, window, profile)?,
            };
            let started = Instant::now();
            let (x, y) = linear.origin();
            let rect = windowed.then(|| WindowRect {
                origin: (window.x - x, window.y - y),
                dims: window.dims(),
            });
            let mut image = self.read_source(&linear, rect, cancel)?;
            if let Some(planes) = planes {
                let (shape, samples) = planes.crop(window);
                image.header.atmosphere = Some(planes.atmosphere);
                image.header.context = Some(shape);
                image.context = samples;
            }
            timings
                .clock()
                .add_wall(timing::READBACK, started.elapsed());
            let tiling = match edge {
                Some(_) => TilingReport::new(tiles, layout.tiles.len()),
                None => job.tiling(),
            };
            Ok(RenderedSource {
                image,
                renderer: "gpu".into(),
                timings: timings.finish(cancel),
                tiling,
            })
        }))
    }

    fn source_context(&self, job: &ContextJob<'_>) -> PipelineResult<Arc<ContextPlanes>> {
        let key = job.sensor.context_key();
        if let Some(planes) = key.and_then(|key| self.sensor.contexts.lock().get(&key).cloned()) {
            return Ok(planes);
        }
        let sensor = job.sensor;
        let haze_level = job.plan.haze.scale.trailing_zeros();
        let planes = sensor.timings().stage(timing::TILE_CONTEXT, || {
            let built = self.build_context(job)?;
            self.read_context(&built.context, haze_level, sensor.cancel())
        })?;
        let planes = Arc::new(planes);
        if let Some(key) = key {
            self.sensor.contexts.lock().put(key, planes.clone());
        }
        Ok(planes)
    }
}
