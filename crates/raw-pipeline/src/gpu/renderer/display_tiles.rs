mod context;
mod source;
mod stitch;

use std::sync::Arc;

use wgpu::TextureUsages;

use super::GpuRenderer;
use super::display::{DisplayRequest, OutputShape};
use super::sensor_stage::{SensorJob, SensorShape, SourcePlan};
use crate::PipelineResult;
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions, RenderedImage};
use crate::gpu::dispatch::{Region, submit_copy};
use crate::gpu::source::LinearSource;
use crate::gpu::texture::{STORAGE_SAMPLED_SRC, texture_2d};
use crate::gpu::timer::RenderTimings;
use crate::ops::lens_blur::{LensBlur, window_margin};
use crate::source::{ContextNeeds, SourceWindow};
use crate::tile::{DisplayLayout, DisplayTile, Rect, TilePlan, TilingReport, support};
use crate::timing;
use context::ContextJob;
pub(super) use context::{Coarse, ContextPlan};
use stitch::Stitch;

const WHOLE_SOURCE_CORE_EDGE: u32 = 2048;
const CONTEXT_USAGE: TextureUsages = STORAGE_SAMPLED_SRC.union(TextureUsages::COPY_DST);

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
        let needs = ContextNeeds::new(&edits.clamped());
        let mut tiles = Self::plan(frame, edits, options, sensor, needs);
        let layout = tiles.layout(frame, edits, sensor, tiles.reach);
        let core_edge = (edge as f32 * tiles.shape.ratio.max(1.0)).ceil() as u32;
        tiles.tiles = layout.tiles(edge);
        tiles.with_cores(core_edge)
    }

    pub fn source(
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        sensor: &SensorShape,
        edge: Option<u32>,
    ) -> Self {
        let needs = ContextNeeds {
            dehaze: true,
            ..ContextNeeds::new(&edits.clamped())
        };
        let mut tiles = Self::plan(frame, edits, options, sensor, needs);
        let reach = window_margin(edits, tiles.working);
        tiles.tiles = vec![tiles.layout(frame, edits, sensor, reach).whole()];
        tiles.with_cores(edge.unwrap_or(WHOLE_SOURCE_CORE_EDGE))
    }

    fn plan(
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        sensor: &SensorShape,
        needs: ContextNeeds,
    ) -> Self {
        let working = sensor.working();
        Self {
            shape: OutputShape::new(&frame.meta, edits, options.max_edge, working),
            tiles: Vec::new(),
            context: ContextPlan::new(needs, working, sensor.chain.align),
            cores: Vec::new(),
            reach: LensBlur::resolve(edits, &options.rasters, working)
                .map_or(0, |blur| blur.scale.margin()),
            working,
        }
    }

    fn with_cores(self, edge: u32) -> Self {
        Self {
            cores: self.context.map_or_else(Vec::new, |plan| plan.cores(edge)),
            ..self
        }
    }

    fn layout<'a>(
        &self,
        frame: &'a RawFrame,
        edits: &'a Edits,
        sensor: &SensorShape,
        reach: u32,
    ) -> DisplayLayout<'a> {
        let meta = &frame.meta;
        DisplayLayout {
            meta,
            edits,
            out: self.shape.full,
            working: self.working,
            target: self.shape.target(self.working),
            halo: support::display_halo(meta.width.max(meta.height) as u32),
            reach,
            align: sensor.chain.align,
        }
    }

    pub fn source_window(&self) -> Rect {
        self.tiles
            .first()
            .map_or(Rect::full(self.working), |tile| tile.window)
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
        (edge, held): (u32, bool),
        timings: RenderTimings<'a>,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<RenderedImage> {
        let plan = SourcePlan::for_render(edits, frame);
        let tile_plan = TilePlan::DisplayTiled { edge, held };
        let (layout, profile, whole, context) = {
            let sensor_plan = if held {
                TilePlan::SensorTiled { edge }
            } else {
                tile_plan
            };
            let job = SensorJob::new(frame, edits, options, plan, sensor_plan, &timings, cancel)?;
            let layout = DisplayTiles::new(frame, edits, options, job.shape(), edge);
            let (whole, profile) = if held {
                (Some(self.sensor_stage(&job)?), None)
            } else {
                let partition = job.shape().chain.tiles(edge);
                (None, self.tiled_noise_profile(&job, &partition)?)
            };
            let context = match &layout.context {
                Some(plan) => Some(timings.stage(timing::TILE_CONTEXT, || {
                    self.build_context(&ContextJob {
                        sensor: &job,
                        options,
                        profile,
                        plan,
                        layout: &layout,
                        whole: whole.as_ref(),
                    })
                })?),
                None => None,
            };
            (layout, profile, whole, context)
        };
        let pooled = self.texture_pool.bytes();
        let mut stitch = Stitch::new(layout.shape.full);
        let mut timings = timings;
        for tile in &layout.tiles {
            let linear = match &whole {
                Some(whole) => self.copy_window(whole, tile.window),
                None => {
                    let job =
                        SensorJob::new(frame, edits, options, plan, tile_plan, &timings, cancel)?;
                    self.sensor_window(&job, tile.window, profile)?
                }
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
            held,
            "gpu display tiles"
        );
        let tiling = TilingReport::new(tile_plan, layout.tiles.len());
        stitch
            .finish(timings)?
            .into_rendered(options, frame.meta.is_raw, tiling, cancel)
    }

    fn copy_window(&self, whole: &LinearSource, window: Rect) -> LinearSource {
        let texture = texture_2d(
            &self.ctx.device,
            "tile-window",
            whole.texture.format(),
            window.dims(),
            1,
            CONTEXT_USAGE,
        );
        submit_copy(
            &self.ctx,
            "tile-window",
            Region::at(&whole.texture, 0, (window.x, window.y)),
            Region::at(&texture, 0, (0, 0)),
            window.dims(),
        );
        LinearSource {
            dims: window.dims(),
            texture: Arc::new(texture),
            window: Some(SourceWindow {
                origin: (window.x, window.y),
                full: whole.dims,
            }),
            ..whole.clone()
        }
    }
}
