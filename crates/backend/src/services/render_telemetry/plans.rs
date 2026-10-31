use raw_pipeline::{TilePlan, TilingReport};

use crate::services::render::RendererKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanKind {
    SinglePass,
    SensorTiled,
    DisplayTiled,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PlanStats {
    pub renderer: RendererKind,
    pub plan: PlanKind,
    pub edge: Option<u32>,
    pub renders: u64,
    pub last_tiles: u32,
}

#[derive(Default)]
pub(super) struct Plans(Vec<PlanStats>);

impl Plans {
    pub fn record(&mut self, renderer: RendererKind, tiling: TilingReport) {
        let (plan, edge) = match tiling.plan {
            TilePlan::SinglePass => (PlanKind::SinglePass, None),
            TilePlan::SensorTiled { edge } => (PlanKind::SensorTiled, Some(edge)),
            TilePlan::DisplayTiled { edge, .. } => (PlanKind::DisplayTiled, Some(edge)),
        };
        let same = |s: &&mut PlanStats| s.renderer == renderer && s.plan == plan && s.edge == edge;
        match self.0.iter_mut().find(same) {
            Some(stats) => {
                stats.renders += 1;
                stats.last_tiles = tiling.tiles;
            }
            None => self.0.push(PlanStats {
                renderer,
                plan,
                edge,
                renders: 1,
                last_tiles: tiling.tiles,
            }),
        }
    }

    pub fn snapshot(&self) -> Vec<PlanStats> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_count_renders_per_renderer_plan_and_edge() {
        let mut plans = Plans::default();
        let display = |edge, tiles| TilingReport {
            plan: TilePlan::DisplayTiled { edge, held: true },
            tiles,
        };
        plans.record(RendererKind::Gpu, TilingReport::SINGLE_PASS);
        plans.record(RendererKind::Gpu, display(2048, 12));
        plans.record(RendererKind::Gpu, display(2048, 9));
        plans.record(RendererKind::Gpu, display(1024, 40));
        plans.record(RendererKind::Cpu, TilingReport::SINGLE_PASS);
        let got: Vec<_> = plans
            .snapshot()
            .into_iter()
            .map(|s| (s.renderer, s.plan, s.edge, s.renders, s.last_tiles))
            .collect();
        let want = [
            (RendererKind::Gpu, PlanKind::SinglePass, None, 1, 1),
            (RendererKind::Gpu, PlanKind::DisplayTiled, Some(2048), 2, 9),
            (RendererKind::Gpu, PlanKind::DisplayTiled, Some(1024), 1, 40),
            (RendererKind::Cpu, PlanKind::SinglePass, None, 1, 1),
        ];
        if got != want {
            panic!("plan stats wrong: {got:?}");
        }
        let json = serde_json::to_value(&plans.snapshot()[1]).unwrap();
        if json["plan"] != "display_tiled" || json["renderer"] != "gpu" || json["edge"] != 2048 {
            panic!("plan stats serialize wrong: {json}");
        }
    }
}
