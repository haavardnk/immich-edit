use super::{Rect, support};
use crate::edits::LensEdits;
use crate::geom::dng_warp::DngWarp;
use crate::ops::lens_ca::ca_scales;
use crate::ops::lens_distortion::LensWarpParams;

#[derive(Debug, Clone)]
pub enum Warp {
    Dng(DngWarp),
    Lens(LensEdits),
}

type SourceMap = Box<dyn Fn(f32, f32) -> [f32; 2]>;

impl Warp {
    pub(super) fn maps(&self, frame: (u32, u32)) -> Vec<SourceMap> {
        match self {
            Self::Dng(dng) => {
                let warp = dng.frame(frame.0 as usize, frame.1 as usize);
                let planes = if dng.is_split() { 3 } else { 1 };
                dng.planes[..planes]
                    .iter()
                    .map(|&plane| Box::new(move |x, y| warp.source_px(&plane, x, y)) as SourceMap)
                    .collect()
            }
            Self::Lens(lens) => {
                let params = LensWarpParams::from_edits(lens, frame.0, frame.1);
                let (red, blue) = ca_scales(lens);
                let (cx, cy) = (frame.0 as f32 * 0.5, frame.1 as f32 * 0.5);
                [1.0, red, blue]
                    .into_iter()
                    .map(|ca| {
                        Box::new(move |x: f32, y: f32| {
                            let (sx, sy) = params.output_px_to_source_px(
                                ca * (x + 0.5 - cx) + cx - 0.5,
                                ca * (y + 0.5 - cy) + cy - 0.5,
                            );
                            [sx, sy]
                        }) as SourceMap
                    })
                    .collect()
            }
        }
    }

    pub(super) fn source(&self, rect: Rect, frame: (u32, u32)) -> Rect {
        self.maps(frame)
            .iter()
            .map(|map| support::warp(rect, frame, map))
            .reduce(Rect::union)
            .unwrap_or(rect)
    }
}

#[derive(Debug, Clone)]
pub struct SensorChain {
    pub sensor: (u32, u32),
    pub working: (u32, u32),
    pub align: u32,
    pub demosaic: u32,
    pub warps: Vec<Warp>,
    pub detail: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SensorTile {
    pub core: Rect,
    pub hist: Rect,
    pub rects: Vec<Rect>,
}

impl SensorTile {
    pub fn input(&self) -> Rect {
        self.rects[0]
    }

    pub fn detail(&self) -> Rect {
        self.rects[self.rects.len() - 1]
    }
}

impl SensorChain {
    pub fn whole(&self) -> SensorTile {
        SensorTile {
            core: Rect::full(self.working),
            hist: Rect::full(self.sensor),
            rects: vec![Rect::full(self.sensor); self.warps.len() + 1],
        }
    }

    pub fn tiles(&self, edge: u32) -> Vec<SensorTile> {
        let edge = edge.div_ceil(self.align).max(1) * self.align;
        Rect::grid(self.working, edge)
            .map(|core| self.tile(core))
            .collect()
    }

    fn tile(&self, core: Rect) -> SensorTile {
        let hist = self.hist_cell(core);
        let needed = self.footprint(core).union(hist);
        let mut rects = vec![needed.expand(self.detail, self.sensor).align(self.align)];
        for warp in self.warps.iter().rev() {
            rects.insert(0, warp.source(rects[0], self.sensor));
        }
        rects[0] = rects[0]
            .expand(self.demosaic, self.sensor)
            .align(self.align);
        SensorTile { core, hist, rects }
    }

    fn scale(&self) -> (f32, f32) {
        (
            self.sensor.0 as f32 / self.working.0 as f32,
            self.sensor.1 as f32 / self.working.1 as f32,
        )
    }

    fn footprint(&self, core: Rect) -> Rect {
        if self.working == self.sensor {
            return core;
        }
        let (sx, sy) = self.scale();
        let (x0, x1) = support::lanczos((core.x, core.right()), sx, self.sensor.0);
        let (y0, y1) = support::lanczos((core.y, core.bottom()), sy, self.sensor.1);
        Rect::bounds(x0, y0, x1, y1)
    }

    fn hist_cell(&self, core: Rect) -> Rect {
        let (sx, sy) = self.scale();
        let edge = |b: u32, working: u32, sensor: u32, scale: f32| {
            if b >= working {
                sensor
            } else {
                (b as f32 * scale).round() as u32 / support::HIST_ALIGN * support::HIST_ALIGN
            }
        };
        Rect::bounds(
            edge(core.x, self.working.0, self.sensor.0, sx),
            edge(core.y, self.working.1, self.sensor.1, sy),
            edge(core.right(), self.working.0, self.sensor.0, sx),
            edge(core.bottom(), self.working.1, self.sensor.1, sy),
        )
    }
}
