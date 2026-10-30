mod stitch;

use std::sync::atomic::Ordering;

use super::GpuRenderer;
use super::display::OutputShape;
use super::sensor_stage::{SensorJob, SensorShape, SourcePlan};
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::gpu::timer::RenderTimings;
use crate::tile::{DisplayLayout, DisplayTile, TilePlan, support};
use stitch::Stitch;

pub(super) struct DisplayTiles {
    pub shape: OutputShape,
    pub tiles: Vec<DisplayTile>,
}

impl DisplayTiles {
    pub fn new(
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        sensor: &SensorShape,
        edge: u32,
    ) -> Self {
        let meta = &frame.meta;
        let working = sensor.working();
        let shape = OutputShape::new(meta, edits, options.max_edge, working);
        let layout = DisplayLayout {
            meta,
            edits,
            out: shape.full,
            working,
            target: shape.target(working),
            halo: support::display_halo(meta.width.max(meta.height) as u32),
            align: sensor.chain.align,
        };
        Self {
            shape,
            tiles: layout.tiles(edge),
        }
    }
}

impl GpuRenderer {
    pub fn display_tiles(&self) -> u64 {
        self.display_tiles.load(Ordering::Relaxed)
    }

    pub(super) fn render_display_tiled<'a>(
        &'a self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        edge: u32,
        timings: RenderTimings<'a>,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        let plan = SourcePlan::for_render(edits, frame);
        let tiling = TilePlan::DisplayTiled { edge };
        let (layout, profile) = {
            let job = SensorJob::new(frame, edits, options, plan, tiling, &timings, cancel)?;
            let layout = DisplayTiles::new(frame, edits, options, job.shape(), edge);
            let partition = job.shape().chain.tiles(edge);
            (layout, self.tiled_noise_profile(&job, &partition)?)
        };
        let pooled = self.texture_pool.bytes();
        let mut stitch = Stitch::new(layout.shape.full);
        let mut timings = timings;
        for tile in &layout.tiles {
            let linear = {
                let job = SensorJob::new(frame, edits, options, plan, tiling, &timings, cancel)?;
                self.sensor_window(&job, tile.window, profile)?
            };
            let shape = layout.shape.tile(tile.rect);
            let display = self.display_chain(&linear, edits, options, shape, timings, cancel)?;
            timings = stitch.place(self.read_display(display, cancel)?, tile)?;
            self.wait_idle(pooled)?;
            crate::cancel::check(cancel)?;
        }
        self.display_tiles
            .fetch_add(layout.tiles.len() as u64, Ordering::Relaxed);
        tracing::debug!(tiles = layout.tiles.len(), edge, "gpu display tiles");
        stitch
            .finish(timings)?
            .into_rendered(options, frame.meta.is_raw, cancel)
    }
}
