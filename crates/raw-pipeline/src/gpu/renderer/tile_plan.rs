use super::{GpuRenderer, SensorShape, SourcePlan};
use crate::cancel::CancelToken;
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
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

struct Tileable {
    fits_single_pass: bool,
    edges: Vec<u32>,
}

impl Tileable {
    fn new(
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        plan: SourcePlan,
        limits: &wgpu::Limits,
    ) -> Option<Self> {
        if plan.presence() && edits.retouch_active() {
            return None;
        }
        let shape = SensorShape::new(frame, &edits.clamped(), options, plan.presence());
        let fits_single_pass = shape.fits(frame, &shape.chain.whole(), limits);
        let (w, h) = shape.working();
        let edges = TILE_EDGES
            .into_iter()
            .filter(|&edge| !fits_single_pass || edge < w.max(h))
            .filter(|&edge| {
                let tiles = shape.chain.tiles(edge);
                tiles.iter().all(|tile| shape.fits(frame, tile, limits))
            })
            .collect();
        Some(Self {
            fits_single_pass,
            edges,
        })
    }
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
            raw.and_then(|(frame, plan)| Tileable::new(frame, edits, options, plan, &limits));
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
        if let Some(tiles) = self.tiles {
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
            .edges
            .iter()
            .map(|&edge| Planned::new(TilePlan::SensorTiled { edge }, estimate))
            .find(|planned| self.within_budget(planned.bytes))
    }

    fn within_budget(&self, bytes: u64) -> bool {
        self.admission.budget().is_none_or(|budget| bytes <= budget)
    }
}
