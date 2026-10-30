use super::{Rect, support};
use crate::edits::{CropRect, Edits};
use crate::frame::FrameMeta;
use crate::source::source_bounds;

const CUBIC_TAPS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayTile {
    pub core: Rect,
    pub rect: Rect,
    pub window: Rect,
}

pub struct DisplayLayout<'a> {
    pub meta: &'a FrameMeta,
    pub edits: &'a Edits,
    pub out: (u32, u32),
    pub working: (u32, u32),
    pub target: Option<(u32, u32)>,
    pub halo: u32,
    pub reach: u32,
    pub align: u32,
}

impl DisplayLayout<'_> {
    pub fn tiles(&self, edge: u32) -> Vec<DisplayTile> {
        Rect::grid(self.out, edge)
            .map(|core| self.tile(core))
            .collect()
    }

    pub fn whole(&self) -> DisplayTile {
        self.tile(Rect::full(self.out))
    }

    fn tile(&self, core: Rect) -> DisplayTile {
        let rect = core.expand(self.halo, self.out);
        let bounds = source_bounds(
            self.meta,
            self.edits,
            self.working,
            normalized(rect, self.halo, self.out),
        );
        let grid = self.target.unwrap_or(self.working);
        let px = |t: f32, n: u32| t * n as f32;
        let sampled = Rect::bounds(
            px(bounds.u.0, grid.0).floor() as u32,
            px(bounds.v.0, grid.1).floor() as u32,
            px(bounds.u.1, grid.0).ceil() as u32,
            px(bounds.v.1, grid.1).ceil() as u32,
        )
        .expand(CUBIC_TAPS, grid);
        let window = match self.target {
            Some(target) => {
                let footprint = |start: u32, end: u32, working: u32, target: u32| {
                    support::lanczos((start, end), working as f32 / target as f32, working)
                };
                let (x0, x1) = footprint(sampled.x, sampled.right(), self.working.0, target.0);
                let (y0, y1) = footprint(sampled.y, sampled.bottom(), self.working.1, target.1);
                Rect::bounds(x0, y0, x1, y1)
            }
            None => sampled,
        };
        DisplayTile {
            core,
            rect,
            window: window.expand(self.reach, self.working).align(self.align),
        }
    }
}

fn normalized(rect: Rect, pad: u32, (w, h): (u32, u32)) -> CropRect {
    let (w, h) = (w as f32, h as f32);
    let pad = pad as f32;
    CropRect {
        x: (rect.x as f32 - pad) / w,
        y: (rect.y as f32 - pad) / h,
        w: (rect.w as f32 + 2.0 * pad) / w,
        h: (rect.h as f32 + 2.0 * pad) / h,
    }
}
