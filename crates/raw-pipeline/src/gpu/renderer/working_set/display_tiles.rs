use super::{DEHAZE_MOMENTS, Levels, RGBA32, WorkingSet};
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::gpu::renderer::SensorShape;
use crate::gpu::renderer::display_tiles::{Coarse, ContextPlan, DisplayTiles};
use crate::gpu::source::SourceExtent;
use crate::source::LinearKind;
use crate::tile::{DisplayTile, Rect, SensorTile};

const GUIDE_TEXTURES: u64 = 2;

impl WorkingSet {
    pub(super) fn display_tiles(
        &mut self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        kind: LinearKind,
        edge: u32,
    ) {
        let edits = edits.clamped();
        let presence = kind == LinearKind::PostWb;
        let shape = SensorShape::new(frame, &edits, options, presence);
        let layout = DisplayTiles::new(frame, &edits, options, &shape, edge);
        let levels = match layout.context {
            Some(_) => Levels::Context,
            None => Levels::Local,
        };
        let sensor_bytes = |tile: &SensorTile| {
            let mut set = self.empty();
            set.sensor_tile(frame, &edits, options, &shape, tile, presence);
            set.bytes
        };
        let extent = |sensor: &SensorTile| {
            let rect = if shape.resamples() {
                sensor.core
            } else {
                sensor.detail()
            };
            SourceExtent {
                dims: rect.dims(),
                full: shape.working(),
            }
        };
        let tile_bytes = |tile: &DisplayTile| {
            let sensor = shape.chain.tile(tile.window);
            let mut set = self.empty();
            set.bytes = sensor_bytes(&sensor);
            let output = (layout.shape.tile(tile.rect), levels);
            set.display(
                &frame.meta,
                (extent(&sensor), kind),
                &edits,
                options,
                output,
            );
            set.bytes
        };
        let core_bytes = |&core: &Rect| {
            let sensor = shape.chain.tile(layout.context_window(core));
            let mut set = self.empty();
            set.bytes = sensor_bytes(&sensor);
            set.spatial(extent(&sensor), &edits, options, Levels::Context);
            set.linear(core.dims(), true);
            set.bytes
        };
        let partition = shape.chain.tiles(edge).iter().map(sensor_bytes).max();
        let tiles = layout.tiles.iter().map(tile_bytes).max();
        let cores = layout.cores.iter().map(core_bytes).max();
        if let Some(plan) = &layout.context {
            self.context(plan);
        }
        self.bytes += partition.max(tiles).max(cores).unwrap_or(0);
    }

    fn context(&mut self, plan: &ContextPlan) {
        if plan.needs.dehaze {
            let lo = plan.lo();
            self.linear(lo, false);
            self.texels(lo, DEHAZE_MOMENTS * RGBA32);
            self.linear(plan.level_dims(plan.atmosphere), false);
        }
        for coarse in plan.coarse() {
            let dims = plan.level_dims(plan.level(coarse));
            self.linear(dims, false);
            self.linear(dims, false);
            if coarse == Coarse::Clarity {
                self.texels(dims, GUIDE_TEXTURES * RGBA32);
            }
        }
    }
}
