mod context;
mod stitch;

use super::GpuRenderer;
use super::display::{DisplayRequest, OutputShape};
use super::sensor_stage::{SensorJob, SensorShape, SourcePlan};
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::gpu::timer::RenderTimings;
use crate::ops::lens_blur::LensBlur;
use crate::tile::{DisplayLayout, DisplayTile, Rect, TilePlan, TilingReport, support};
use crate::timing;
use context::ContextJob;
pub(super) use context::{Coarse, ContextPlan};
use stitch::Stitch;

pub(super) struct DisplayTiles {
    pub shape: OutputShape,
    pub tiles: Vec<DisplayTile>,
    pub context: Option<ContextPlan>,
    pub cores: Vec<Rect>,
    reach: u32,
    working: (u32, u32),
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
        let reach = LensBlur::resolve(edits, &options.rasters, working)
            .map_or(0, |blur| blur.scale.margin());
        let layout = DisplayLayout {
            meta,
            edits,
            out: shape.full,
            working,
            target: shape.target(working),
            halo: support::display_halo(meta.width.max(meta.height) as u32),
            reach,
            align: sensor.chain.align,
        };
        let context = ContextPlan::new(&edits.clamped(), working, sensor.chain.align);
        let core_edge = (edge as f32 * shape.ratio.max(1.0)).ceil() as u32;
        Self {
            shape,
            tiles: layout.tiles(edge),
            cores: context.map_or_else(Vec::new, |plan| plan.cores(core_edge)),
            context,
            reach,
            working,
        }
    }

    pub fn context_window(&self, core: Rect) -> Rect {
        core.expand(self.reach, self.working)
    }

    pub fn windows(&self) -> impl Iterator<Item = Rect> + '_ {
        let context = self.cores.iter().map(|&core| self.context_window(core));
        context.chain(self.tiles.iter().map(|tile| tile.window))
    }
}

impl GpuRenderer {
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
        let tile_plan = TilePlan::DisplayTiled { edge };
        let (layout, profile, context) = {
            let job = SensorJob::new(frame, edits, options, plan, tile_plan, &timings, cancel)?;
            let layout = DisplayTiles::new(frame, edits, options, job.shape(), edge);
            let partition = job.shape().chain.tiles(edge);
            let profile = self.tiled_noise_profile(&job, &partition)?;
            let context = match &layout.context {
                Some(plan) => Some(timings.stage(timing::TILE_CONTEXT, || {
                    self.build_context(&ContextJob {
                        sensor: &job,
                        options,
                        profile,
                        plan,
                        layout: &layout,
                    })
                })?),
                None => None,
            };
            (layout, profile, context)
        };
        let pooled = self.texture_pool.bytes();
        let mut stitch = Stitch::new(layout.shape.full);
        let mut timings = timings;
        for tile in &layout.tiles {
            let linear = {
                let job = SensorJob::new(frame, edits, options, plan, tile_plan, &timings, cancel)?;
                self.sensor_window(&job, tile.window, profile)?
            };
            let request = DisplayRequest {
                source: &linear,
                edits,
                opts: options,
                shape: layout.shape.tile(tile),
                context: context.as_ref().map(|tiles| &tiles.context),
            };
            let display = self.display_chain(request, timings, cancel)?;
            timings = stitch.place(self.read_display(display, cancel)?, tile)?;
            self.wait_idle(pooled)?;
            crate::cancel::check(cancel)?;
        }
        tracing::debug!(
            tiles = layout.tiles.len(),
            context = layout.cores.len(),
            edge,
            "gpu display tiles"
        );
        let tiling = TilingReport {
            plan: tile_plan,
            tiles: layout.tiles.len() as u32,
        };
        stitch
            .finish(timings)?
            .into_rendered(options, frame.meta.is_raw, tiling, cancel)
    }
}
