use super::{DEHAZE_MOMENTS, Levels, RGBA16, RGBA32, WorkingSet, host};
use crate::edits::Edits;
use crate::frame::{RawFrame, RenderOptions};
use crate::gpu::readback::readback_size;
use crate::gpu::renderer::display_tiles::{Coarse, ContextPlan, DisplayTiles};
use crate::gpu::renderer::{SensorShape, display_depth};
use crate::gpu::source::SourceExtent;
use crate::source::{ContextNeeds, LinearKind};
use crate::tile::{DisplayTile, Rect, SensorTile, TilePlan};

const GUIDE_TEXTURES: u64 = 2;

struct Tiled<'a> {
    frame: &'a RawFrame,
    edits: &'a Edits,
    options: &'a RenderOptions,
    shape: SensorShape,
    presence: bool,
}

impl Tiled<'_> {
    fn sensor(&self, base: &WorkingSet, tile: &SensorTile) -> WorkingSet {
        let mut set = base.empty();
        set.sensor_tile(
            self.frame,
            self.edits,
            self.options,
            &self.shape,
            tile,
            self.presence,
        );
        set
    }

    fn extent(&self, sensor: &SensorTile) -> SourceExtent {
        let rect = if self.shape.resamples() {
            sensor.core
        } else {
            sensor.detail
        };
        SourceExtent {
            dims: rect.dims(),
            full: self.shape.working(),
        }
    }

    fn partition(&self, base: &WorkingSet, edge: u32) -> Option<u64> {
        let tiles = self.shape.chain.tiles(edge);
        tiles.iter().map(|tile| self.sensor(base, tile).bytes).max()
    }

    fn core(&self, base: &WorkingSet, layout: &DisplayTiles, core: Rect) -> u64 {
        let sensor = self.shape.chain.tile(layout.context_window(core));
        let mut set = self.sensor(base, &sensor);
        set.context_core(self.extent(&sensor), self.edits, self.options, core);
        set.bytes
    }
}

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
        let tiled = Tiled {
            frame,
            edits: &edits,
            options,
            shape: SensorShape::new(frame, &edits, options, presence),
            presence,
        };
        let layout = DisplayTiles::new(frame, &edits, options, &tiled.shape, edge);
        let levels = match layout.context {
            Some(_) => Levels::Context,
            None => Levels::Local,
        };
        let tile_bytes = |tile: &DisplayTile| {
            let sensor = tiled.shape.chain.tile(tile.window);
            let mut set = tiled.sensor(self, &sensor);
            let output = (layout.shape.tile(tile), levels);
            set.display(
                &frame.meta,
                (tiled.extent(&sensor), kind),
                &edits,
                options,
                output,
            );
            set.bytes
        };
        let partition = tiled.partition(self, edge);
        let tiles = layout.tiles.iter().map(tile_bytes).max();
        let cores = layout
            .cores
            .iter()
            .map(|&core| tiled.core(self, &layout, core))
            .max();
        if let Some(plan) = &layout.context {
            self.context(plan);
        }
        self.bytes += partition.max(tiles).max(cores).unwrap_or(0);
        let (out, depth) = (layout.shape.dims(), display_depth(options));
        let stitched = host::held(out, depth) + self.upload.get();
        self.host = host::display(out, depth, options).max(stitched);
    }

    pub(super) fn source_window(
        &mut self,
        frame: &RawFrame,
        edits: &Edits,
        options: &RenderOptions,
        tiles: TilePlan,
    ) {
        let edits = edits.clamped();
        let tiled = Tiled {
            frame,
            edits: &edits,
            options,
            shape: SensorShape::new(frame, &edits, options, true),
            presence: true,
        };
        let edge = tiles.display_edge();
        let layout = DisplayTiles::source(frame, &edits, options, &tiled.shape, edge);
        let window = layout.source_window();
        let work = match edge {
            Some(edge) => {
                let sensor = tiled.shape.chain.tile(window);
                let cores = layout
                    .cores
                    .iter()
                    .map(|&core| tiled.core(self, &layout, core));
                cores
                    .chain(tiled.partition(self, edge))
                    .chain([tiled.sensor(self, &sensor).bytes])
                    .max()
                    .unwrap_or(0)
            }
            None => {
                let mut sensor = self.empty();
                let extent = sensor.sensor(frame, &edits, options, true, tiles);
                let cores = layout.cores.iter().map(|&core| {
                    let window = layout.context_window(core);
                    let mut copy = self.empty();
                    copy.linear(extent.full, true);
                    copy.linear(window.dims(), false);
                    let extent = SourceExtent {
                        dims: window.dims(),
                        full: extent.full,
                    };
                    copy.context_core(extent, &edits, options, core);
                    copy.bytes
                });
                cores.chain([sensor.bytes]).max().unwrap_or(0)
            }
        };
        self.host = host::source(window.dims());
        if let Some(plan) = &layout.context {
            self.context(plan);
            let lo = plan.lo();
            self.bytes += readback_size(lo.0, lo.1, RGBA32 as u32);
            self.host += host::context(plan);
        }
        self.bytes += work + readback_size(window.w, window.h, RGBA16 as u32);
    }

    fn context_core(
        &mut self,
        extent: SourceExtent,
        edits: &Edits,
        options: &RenderOptions,
        core: Rect,
    ) {
        let needs = ContextNeeds::new(edits);
        if needs.dehaze {
            self.linear(extent.dims, false);
        }
        self.lens_blur(extent, edits, options);
        if needs.shadows && needs.presence {
            self.linear(extent.dims, false);
        }
        self.linear(core.dims(), true);
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
