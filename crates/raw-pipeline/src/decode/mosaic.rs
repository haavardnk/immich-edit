use crate::sensor_sample::cfa_block;
use rawler::RawImage;
use rayon::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveArea {
    pub origin: [usize; 2],
    pub size: [usize; 2],
}

impl ActiveArea {
    pub(crate) fn of(raw: &RawImage) -> Self {
        match raw.active_area {
            Some(area) => Self {
                origin: [area.p.x, area.p.y],
                size: [area.d.w, area.d.h],
            },
            None => Self {
                origin: [0, 0],
                size: [raw.width, raw.height],
            },
        }
    }

    pub(crate) fn fits(&self, width: usize, height: usize) -> bool {
        self.origin[0] + self.size[0] <= width && self.origin[1] + self.size[1] <= height
    }
}

pub(crate) struct Cfa {
    dim: usize,
    table: Vec<usize>,
}

impl Cfa {
    pub(crate) fn new(pattern: &str) -> Self {
        let (dim, table) = cfa_block(pattern);
        Self { dim, table }
    }

    pub(crate) fn dim(&self) -> usize {
        self.dim
    }

    pub(crate) fn color(&self, x: usize, y: usize) -> usize {
        self.table[(y % self.dim) * self.dim + x % self.dim]
    }
}

pub(crate) struct Mosaic<'a> {
    data: &'a mut [f32],
    stride: usize,
    origin: [usize; 2],
    pub width: usize,
    pub height: usize,
    pub cfa: Cfa,
}

impl<'a> Mosaic<'a> {
    pub(crate) fn whole(data: &'a mut [f32], width: usize, height: usize, pattern: &str) -> Self {
        let area = ActiveArea {
            origin: [0, 0],
            size: [width, height],
        };
        Self::window(data, width, area, pattern)
    }

    pub(crate) fn window(
        data: &'a mut [f32],
        stride: usize,
        area: ActiveArea,
        pattern: &str,
    ) -> Self {
        Self {
            data,
            stride,
            origin: area.origin,
            width: area.size[0],
            height: area.size[1],
            cfa: Cfa::new(pattern),
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        (self.origin[1] + y) * self.stride + self.origin[0] + x
    }

    pub(crate) fn get(&self, x: usize, y: usize) -> f32 {
        self.data[self.index(x, y)]
    }

    pub(crate) fn set(&mut self, x: usize, y: usize, value: f32) {
        let i = self.index(x, y);
        self.data[i] = value;
    }

    pub(crate) fn par_rows_mut(
        &mut self,
    ) -> (
        impl IndexedParallelIterator<Item = (usize, &mut [f32])> + '_,
        &Cfa,
    ) {
        let [x0, y0] = self.origin;
        let width = self.width;
        let rows = self
            .data
            .par_chunks_mut(self.stride.max(1))
            .skip(y0)
            .take(self.height)
            .map(move |row| &mut row[x0..x0 + width])
            .enumerate();
        (rows, &self.cfa)
    }
}
