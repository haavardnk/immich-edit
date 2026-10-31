use super::opcodes::OpcodeList;
use crate::decode::mosaic::Mosaic;
use crate::geom::dng_warp::{DngWarp, WarpFrame};
use rayon::prelude::*;

const MAX_GAIN: f32 = 1024.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MapArea {
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GainMap {
    pub area: MapArea,
    pub plane: u32,
    pub planes: u32,
    pub pitch: [u32; 2],
    pub points: [usize; 2],
    pub spacing: [f64; 2],
    pub origin: [f64; 2],
    pub map_planes: usize,
    pub gains: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct VignetteRadial {
    pub k: [f64; 5],
    pub center: [f64; 2],
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Gain {
    Map(GainMap),
    Radial(VignetteRadial),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct StagedGain {
    pub list: OpcodeList,
    pub after_warp: bool,
    pub gain: Gain,
}

#[derive(Debug, Default)]
pub(super) struct Gains {
    pub(super) ops: Vec<StagedGain>,
}

struct RadialFrame {
    center: [f64; 2],
    inv_radius2: f64,
    k: [f64; 5],
}

enum Prepared<'a> {
    Map(&'a GainMap),
    Radial(RadialFrame),
}

struct PreparedGain<'a> {
    list: OpcodeList,
    after_warp: bool,
    op: Prepared<'a>,
}

struct Field<'a> {
    ops: Vec<PreparedGain<'a>>,
    size: [f64; 2],
    warp: Option<(WarpFrame, &'a DngWarp)>,
}

struct Point {
    centre: [f64; 2],
    cell: [usize; 2],
}

impl Point {
    fn at(x: usize, y: usize) -> Self {
        Self {
            centre: [x as f64 + 0.5, y as f64 + 0.5],
            cell: [x, y],
        }
    }

    fn warped(frame: &WarpFrame, warp: &DngWarp, color: usize, x: usize, y: usize) -> Option<Self> {
        let plane = &warp.planes[color.min(2)];
        let [dx, dy] = frame.destination(plane, x as f32 + 0.5, y as f32 + 0.5);
        let inside = dx.is_finite() && dy.is_finite() && dx >= 0.0 && dy >= 0.0;
        inside.then(|| Self {
            centre: [f64::from(dx), f64::from(dy)],
            cell: [dx as usize, dy as usize],
        })
    }
}

impl GainMap {
    pub fn is_identity(&self) -> bool {
        self.gains.iter().all(|g| *g == 1.0)
    }

    fn covers(&self, [x, y]: [usize; 2]) -> bool {
        let a = self.area;
        let (Ok(x), Ok(y)) = (u32::try_from(x), u32::try_from(y)) else {
            return false;
        };
        (a.top..a.bottom).contains(&y)
            && (a.left..a.right).contains(&x)
            && (y - a.top).is_multiple_of(self.pitch[0])
            && (x - a.left).is_multiple_of(self.pitch[1])
    }

    fn map_plane(&self, plane: usize) -> Option<usize> {
        let first = self.plane as usize;
        let range = first..first.saturating_add(self.planes as usize);
        range
            .contains(&plane)
            .then(|| plane.min(self.map_planes - 1))
    }

    fn sample(&self, rel: [f64; 2], map_plane: usize) -> f32 {
        let (v0, v1, fv) = axis(rel[0], self.origin[0], self.spacing[0], self.points[0]);
        let (h0, h1, fh) = axis(rel[1], self.origin[1], self.spacing[1], self.points[1]);
        let at = |v: usize, h: usize| -> f64 {
            f64::from(self.gains[(v * self.points[1] + h) * self.map_planes + map_plane])
        };
        let top = at(v0, h0) + (at(v0, h1) - at(v0, h0)) * fh;
        let bottom = at(v1, h0) + (at(v1, h1) - at(v1, h0)) * fh;
        (top + (bottom - top) * fv) as f32
    }
}

fn axis(rel: f64, origin: f64, spacing: f64, points: usize) -> (usize, usize, f64) {
    let last = points - 1;
    if last == 0 {
        return (0, 0, 0.0);
    }
    let f = (rel - origin) / spacing;
    if f <= 0.0 {
        return (0, 0, 0.0);
    }
    if f >= last as f64 {
        return (last, last, 0.0);
    }
    let i = f.floor() as usize;
    (i, i + 1, f - i as f64)
}

impl VignetteRadial {
    pub fn is_identity(&self) -> bool {
        self.k.iter().all(|k| *k == 0.0)
    }

    fn frame(&self, size: [f64; 2]) -> RadialFrame {
        let [w, h] = size;
        let cx = self.center[0] * w;
        let cy = self.center[1] * h;
        let reach_x = cx.max(w - cx);
        let reach_y = cy.max(h - cy);
        let radius2 = reach_x * reach_x + reach_y * reach_y;
        RadialFrame {
            center: [cx, cy],
            inv_radius2: if radius2 > 0.0 { radius2.recip() } else { 0.0 },
            k: self.k,
        }
    }
}

impl RadialFrame {
    fn gain(&self, [x, y]: [f64; 2]) -> f32 {
        let dx = x - self.center[0];
        let dy = y - self.center[1];
        let r2 = (dx * dx + dy * dy) * self.inv_radius2;
        let [k0, k1, k2, k3, k4] = self.k;
        let poly = k0 + r2 * (k1 + r2 * (k2 + r2 * (k3 + r2 * k4)));
        (1.0 + r2 * poly).max(0.0) as f32
    }
}

impl PreparedGain<'_> {
    fn gain(&self, point: &Point, size: [f64; 2], plane: usize) -> Option<f32> {
        match &self.op {
            Prepared::Radial(frame) => Some(frame.gain(point.centre)),
            Prepared::Map(map) if map.covers(point.cell) => {
                let rel = [point.centre[1] / size[1], point.centre[0] / size[0]];
                map.map_plane(plane).map(|mp| map.sample(rel, mp))
            }
            Prepared::Map(_) => None,
        }
    }
}

impl Field<'_> {
    fn gain(&self, x: usize, y: usize, color: usize, mosaic: bool) -> f32 {
        let direct = Point::at(x, y);
        let warped = self
            .warp
            .as_ref()
            .filter(|_| self.ops.iter().any(|op| op.after_warp))
            .and_then(|(frame, warp)| Point::warped(frame, warp, color, x, y));
        self.ops
            .iter()
            .filter_map(|op| {
                let point = match op.after_warp {
                    true => warped.as_ref()?,
                    false => &direct,
                };
                let plane = match (mosaic, op.list) {
                    (true, OpcodeList::Two) => 0,
                    _ => color,
                };
                op.gain(point, self.size, plane)
            })
            .fold(1.0, |acc, g| (acc * g).min(MAX_GAIN))
    }
}

impl Gains {
    pub(super) fn push(&mut self, gain: StagedGain) {
        self.ops.push(gain);
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    fn field<'a>(&'a self, width: usize, height: usize, warp: Option<&'a DngWarp>) -> Field<'a> {
        let size = [width as f64, height as f64];
        let ops = self
            .ops
            .iter()
            .map(|staged| PreparedGain {
                list: staged.list,
                after_warp: staged.after_warp,
                op: match &staged.gain {
                    Gain::Map(map) => Prepared::Map(map),
                    Gain::Radial(radial) => Prepared::Radial(radial.frame(size)),
                },
            })
            .collect();
        Field {
            ops,
            size,
            warp: warp.map(|w| (w.frame(width, height), w)),
        }
    }

    pub(super) fn apply_mosaic(&self, mosaic: &mut Mosaic, warp: Option<&DngWarp>) {
        if self.is_empty() {
            return;
        }
        let field = self.field(mosaic.width, mosaic.height, warp);
        let (rows, cfa) = mosaic.par_rows_mut();
        rows.for_each(|(y, row)| {
            row.iter_mut().enumerate().for_each(|(x, v)| {
                *v *= field.gain(x, y, cfa.color(x, y), true);
            });
        });
    }

    pub(super) fn apply_rgb(
        &self,
        rgb: &mut [f32],
        width: usize,
        height: usize,
        warp: Option<&DngWarp>,
    ) {
        if self.is_empty() || width == 0 {
            return;
        }
        let field = self.field(width, height, warp);
        rgb.par_chunks_exact_mut(width * 3)
            .enumerate()
            .for_each(|(y, row)| {
                row.iter_mut().enumerate().for_each(|(i, v)| {
                    *v *= field.gain(i / 3, y, i % 3, false);
                });
            });
    }
}

#[cfg(test)]
mod tests;
