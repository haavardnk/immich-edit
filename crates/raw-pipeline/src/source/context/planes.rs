use super::grid::BILINEAR_SLACK;
use super::{ContextGrid, ContextPart, SourceContext};
use crate::tile::Rect;

pub(crate) struct ContextPlane {
    pub part: ContextPart,
    pub level: u32,
    pub dims: (u32, u32),
    pub samples: Vec<f32>,
}

impl ContextPlane {
    fn crop(&self, window: Rect) -> Rect {
        let span = |start: u32, end: u32, len: u32| {
            let lo = (start >> self.level)
                .saturating_sub(BILINEAR_SLACK)
                .min(len - 1);
            let hi = (end.div_ceil(1 << self.level) + BILINEAR_SLACK).min(len);
            (lo, hi.max(lo + 1))
        };
        let (x0, x1) = span(window.x, window.right(), self.dims.0);
        let (y0, y1) = span(window.y, window.bottom(), self.dims.1);
        Rect::bounds(x0, y0, x1, y1)
    }
}

pub(crate) struct ContextPlanes {
    pub atmosphere: [f32; 3],
    pub planes: Vec<ContextPlane>,
}

impl ContextPlanes {
    pub fn crop(&self, window: Rect) -> (SourceContext, Vec<f32>) {
        let mut shape = SourceContext::default();
        let mut samples = Vec::new();
        for plane in &self.planes {
            let crop = plane.crop(window);
            let channels = plane.part.channels();
            let stride = plane.dims.0 as usize * channels;
            let (x0, x1) = (crop.x as usize * channels, crop.right() as usize * channels);
            for y in crop.y as usize..crop.bottom() as usize {
                samples.extend_from_slice(&plane.samples[y * stride + x0..y * stride + x1]);
            }
            let grid = ContextGrid {
                level: plane.level,
                origin: (crop.x, crop.y),
                dims: crop.dims(),
            };
            shape.set(plane.part, grid);
        }
        (shape, samples)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plane(level: u32, dims: (u32, u32)) -> ContextPlane {
        ContextPlane {
            part: ContextPart::Haze,
            level,
            dims,
            samples: (0..dims.0 * dims.1 * 2).map(|i| i as f32).collect(),
        }
    }

    #[test]
    fn a_crop_covers_every_bilinear_tap_of_the_window() {
        let window = Rect {
            x: 401,
            y: 37,
            w: 250,
            h: 90,
        };
        let crop = plane(2, (300, 200)).crop(window);
        let tap = |px: u32| ((px as f32 + 0.5) / 4.0 - 0.5).floor() as i64;
        let (lo, hi) = (tap(window.x), tap(window.right() - 1) + 1);
        if i64::from(crop.x) > lo || i64::from(crop.right()) <= hi {
            panic!("{crop:?} misses taps {lo}..={hi}");
        }
        let edge = plane(2, (300, 200)).crop(Rect { x: 1100, ..window });
        if edge.right() != 300 || edge.w == 0 {
            panic!("a crop past the level edge was not clamped: {edge:?}");
        }
    }

    #[test]
    fn a_crop_copies_its_rows_from_the_plane() {
        let planes = ContextPlanes {
            atmosphere: [0.9; 3],
            planes: vec![plane(2, (30, 20))],
        };
        let (shape, samples) = planes.crop(Rect {
            x: 40,
            y: 20,
            w: 8,
            h: 4,
        });
        let Some(grid) = shape.get(ContextPart::Haze) else {
            panic!("the haze plane was not cropped");
        };
        let first = (grid.origin.1 * 30 + grid.origin.0) * 2;
        let second_row = ((grid.origin.1 + 1) * 30 + grid.origin.0) * 2;
        let width = grid.dims.0 as usize * 2;
        if grid.origin != (8, 3)
            || samples.len() as u64 != shape.samples()
            || samples[0] != first as f32
            || samples[width] != second_row as f32
        {
            panic!("crop {grid:?} copied {:?}", &samples[..4]);
        }
    }
}
