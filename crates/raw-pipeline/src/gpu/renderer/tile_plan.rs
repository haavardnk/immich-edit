use super::display_tiles::DisplayTiles;
use super::{GpuRenderer, SensorShape, SourcePlan, presence_display};
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{PreviewMode, RawFrame, RenderOptions};
use crate::geom;
use crate::tile::TilePlan;
use crate::{PipelineError, PipelineResult};

const TILE_EDGES: [u32; 4] = [4096, 2048, 1024, 512];

#[derive(Clone, Copy)]
struct Planned {
    tiles: TilePlan,
    bytes: u64,
}

impl Planned {
    fn new(tiles: TilePlan, estimate: &impl Fn(TilePlan) -> u64) -> Self {
        Self {
            tiles,
            bytes: estimate(tiles),
        }
    }
}

struct Tileable<'a> {
    frame: &'a RawFrame,
    edits: &'a Edits,
    options: &'a RenderOptions,
    limits: wgpu::Limits,
    shape: SensorShape,
    out: (u32, u32),
    chain_fits: bool,
    fits_single_pass: bool,
    display: bool,
}

impl<'a> Tileable<'a> {
    fn new(
        frame: &'a RawFrame,
        edits: &'a Edits,
        options: &'a RenderOptions,
        plan: SourcePlan,
        limits: wgpu::Limits,
    ) -> Option<Self> {
        if plan.presence() && edits.retouch_active() {
            return None;
        }
        let shape = SensorShape::new(frame, &edits.clamped(), options, plan.presence());
        let meta = &frame.meta;
        let dims = (meta.width as u32, meta.height as u32);
        let out = geom::display_out_dims(meta.orientation, edits, dims, options.max_edge);
        let edge = |(w, h): (u32, u32)| w.max(h) <= limits.max_texture_dimension_2d;
        let chain_fits = (!plan.displays() || edge(out)) && edge(shape.working());
        let fits_single_pass = chain_fits && shape.fits(frame, &shape.chain.whole(), &limits);
        Some(Self {
            frame,
            edits,
            options,
            limits,
            shape,
            out,
            chain_fits,
            fits_single_pass,
            display: plan.displays() && display_tileable(edits, options),
        })
    }

    fn candidates(&self) -> impl Iterator<Item = TilePlan> + '_ {
        let (w, h) = self.shape.working();
        let sensor = TILE_EDGES
            .into_iter()
            .filter(|_| self.chain_fits)
            .filter(move |&edge| !self.fits_single_pass || edge < w.max(h))
            .filter(|&edge| self.sensor_fits(edge))
            .map(|edge| TilePlan::SensorTiled { edge });
        let display = TILE_EDGES
            .into_iter()
            .filter(|_| self.display)
            .filter(|&edge| edge < self.out.0.max(self.out.1))
            .filter(|&edge| self.display_fits(edge))
            .map(|edge| TilePlan::DisplayTiled { edge });
        sensor.chain(display)
    }

    fn sensor_fits(&self, edge: u32) -> bool {
        let tiles = self.shape.chain.tiles(edge);
        tiles
            .iter()
            .all(|tile| self.shape.fits(self.frame, tile, &self.limits))
    }

    fn display_fits(&self, edge: u32) -> bool {
        let layout = DisplayTiles::new(self.frame, self.edits, self.options, &self.shape, edge);
        layout.tiles.iter().all(|tile| {
            let sensor = self.shape.chain.tile(tile.window);
            tile.rect.w.max(tile.rect.h) <= self.limits.max_texture_dimension_2d
                && self.shape.fits(self.frame, &sensor, &self.limits)
        })
    }
}

fn display_tileable(edits: &Edits, options: &RenderOptions) -> bool {
    !presence_display(edits)
        && !options.histogram
        && !options.scopes
        && options.preview_mode == PreviewMode::None
}

impl GpuRenderer {
    pub(super) fn run_planned<T>(
        &self,
        raw: Option<(&RawFrame, SourcePlan)>,
        edits: &Edits,
        options: &RenderOptions,
        estimate: impl Fn(TilePlan) -> u64,
        render: impl Fn(TilePlan) -> PipelineResult<T>,
        cancel: Option<&CancelToken>,
    ) -> PipelineResult<T> {
        let limits = self.ctx.device.limits();
        let tileable =
            raw.and_then(|(frame, plan)| Tileable::new(frame, edits, options, plan, limits));
        let planned = self.plan(tileable.as_ref(), &estimate);
        let run = |planned: Planned| {
            let _admitted = self.admit(planned.bytes, cancel)?;
            render(planned.tiles)
        };
        match run(planned) {
            Err(PipelineError::OutOfMemory(e)) if planned.tiles == TilePlan::SinglePass => {
                match tileable.and_then(|tileable| self.tiled(&tileable, &estimate)) {
                    Some(retry) => run(retry),
                    None => Err(PipelineError::OutOfMemory(e)),
                }
            }
            result => result,
        }
    }

    fn plan(&self, tileable: Option<&Tileable>, estimate: &impl Fn(TilePlan) -> u64) -> Planned {
        let Some(tileable) = tileable else {
            return Planned::new(TilePlan::SinglePass, estimate);
        };
        let forced = self
            .tiles
            .filter(|tiles| tileable.display || !matches!(tiles, TilePlan::DisplayTiled { .. }));
        if let Some(tiles) = forced {
            return Planned::new(tiles, estimate);
        }
        let single = Planned::new(TilePlan::SinglePass, estimate);
        if tileable.fits_single_pass && self.within_budget(single.bytes) {
            return single;
        }
        self.tiled(tileable, estimate).unwrap_or(single)
    }

    fn tiled(&self, tileable: &Tileable, estimate: &impl Fn(TilePlan) -> u64) -> Option<Planned> {
        tileable
            .candidates()
            .map(|tiles| Planned::new(tiles, estimate))
            .find(|planned| self.within_budget(planned.bytes))
    }

    fn within_budget(&self, bytes: u64) -> bool {
        self.admission.budget().is_none_or(|budget| bytes <= budget)
    }
}
