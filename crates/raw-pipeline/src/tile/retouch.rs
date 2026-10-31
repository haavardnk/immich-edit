use super::{Rect, SensorTile};
use crate::edits::{Edits, RetouchStroke};
use crate::frame::OrientFlips;
use crate::ops::retouch::{StrokeGeom, stroke_geometry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrokeReach {
    pub index: usize,
    pub write: Rect,
    pub read: Rect,
}

impl StrokeReach {
    pub(crate) fn new(index: usize, geom: &StrokeGeom, frame: (u32, u32)) -> Self {
        let b = &geom.bbox;
        let write = Rect::bounds(b.x0 as u32, b.y0 as u32, b.x1 as u32, b.y1 as u32);
        let (x0, x1) = taps(b.x0, b.x1, geom.off_x, frame.0);
        let (y0, y1) = taps(b.y0, b.y1, geom.off_y, frame.1);
        Self {
            index,
            write,
            read: write.union(Rect::bounds(x0, y0, x1, y1)),
        }
    }

    pub fn all(edits: &Edits, frame: (u32, u32), orientation: OrientFlips) -> Vec<Self> {
        let (w, h) = (frame.0 as usize, frame.1 as usize);
        edits
            .retouch
            .iter()
            .enumerate()
            .filter(|(_, stroke)| stroke.is_effective())
            .filter_map(|(index, stroke)| {
                let geom = stroke_geometry(stroke, w, h, orientation)?;
                Some(Self::new(index, &geom, frame))
            })
            .collect()
    }
}

fn taps(lo: usize, hi: usize, offset: f32, extent: u32) -> (u32, u32) {
    let floor = |px: usize| (px as f64 + f64::from(offset)).floor() as i64;
    let extent = i64::from(extent);
    let first = (floor(lo) - 2).clamp(0, extent - 1);
    let end = (floor(hi - 1) + 4).clamp(1, extent);
    (first as u32, end as u32)
}

pub fn closure(strokes: &[StrokeReach], rect: Rect) -> (Rect, Vec<usize>) {
    let mut needed = vec![rect];
    let mut included = Vec::new();
    for stroke in strokes.iter().rev() {
        if needed.iter().any(|r| r.meets(stroke.write)) {
            needed.push(stroke.read);
            included.push(stroke.index);
        }
    }
    included.reverse();
    let window = needed.into_iter().reduce(Rect::union).unwrap_or(rect);
    (window, included)
}

pub(crate) fn tile_strokes<'e>(
    edits: &'e Edits,
    tile: &SensorTile,
    frame: (u32, u32),
    orientation: OrientFlips,
) -> Vec<(&'e RetouchStroke, StrokeGeom)> {
    let (w, h) = (frame.0 as usize, frame.1 as usize);
    tile.strokes
        .iter()
        .filter_map(|&index| {
            let stroke = edits.retouch.get(index)?;
            Some((stroke, stroke_geometry(stroke, w, h, orientation)?))
        })
        .collect()
}
