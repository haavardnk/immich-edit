use super::mosaic::{Cfa, Mosaic};
use rayon::prelude::*;

const RING_RADIUS: isize = 2;
const MAX_RING: usize = 24;
const OUTLIER_RATIO: f32 = 2.0;
const OUTLIER_FLOOR: f32 = 0.02;
const LIGHT_SPREAD: f32 = 0.05;
const MIN_RING: usize = 3;

struct Rings {
    dim: usize,
    offsets: Vec<Vec<[isize; 2]>>,
}

impl Rings {
    fn new(cfa: &Cfa) -> Self {
        let dim = cfa.dim();
        let window = || {
            (-RING_RADIUS..=RING_RADIUS)
                .flat_map(|dy| (-RING_RADIUS..=RING_RADIUS).map(move |dx| [dx, dy]))
        };
        let offsets = (0..dim * dim)
            .map(|phase| {
                let px = phase % dim;
                let py = phase / dim;
                let base = dim * 4;
                let color = cfa.color(px, py);
                window()
                    .filter(|&[dx, dy]| (dx, dy) != (0, 0))
                    .filter(|&[dx, dy]| {
                        let x = (base as isize + px as isize + dx) as usize;
                        let y = (base as isize + py as isize + dy) as usize;
                        cfa.color(x, y) == color
                    })
                    .collect()
            })
            .collect();
        Self { dim, offsets }
    }

    fn sites(&self, mosaic: &Mosaic, x: usize, y: usize) -> impl Iterator<Item = [usize; 2]> {
        let w = mosaic.width as isize;
        let h = mosaic.height as isize;
        self.offsets[(y % self.dim) * self.dim + x % self.dim]
            .iter()
            .map(move |&[dx, dy]| [x as isize + dx, y as isize + dy])
            .filter(move |&[sx, sy]| (0..w).contains(&sx) && (0..h).contains(&sy))
            .map(|[sx, sy]| [sx as usize, sy as usize])
    }

    fn values(
        &self,
        mosaic: &Mosaic,
        x: usize,
        y: usize,
        skip: impl Fn([usize; 2]) -> bool,
    ) -> Ring {
        let mut ring = Ring::default();
        self.sites(mosaic, x, y)
            .filter(|&site| !skip(site))
            .for_each(|[sx, sy]| ring.push(mosaic.get(sx, sy)));
        ring
    }
}

#[derive(Default)]
struct Ring {
    values: [f32; MAX_RING],
    len: usize,
}

impl Ring {
    fn push(&mut self, value: f32) {
        if self.len < MAX_RING {
            self.values[self.len] = value;
            self.len += 1;
        }
    }

    fn slice(&self) -> &[f32] {
        &self.values[..self.len]
    }

    fn median(&mut self) -> Option<f32> {
        let len = self.len;
        let values = &mut self.values[..len];
        if values.is_empty() {
            return None;
        }
        values.sort_unstable_by(f32::total_cmp);
        let mid = len / 2;
        Some(if len % 2 == 1 {
            values[mid]
        } else {
            0.5 * (values[mid - 1] + values[mid])
        })
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Outlier {
    Hot,
    Dead,
}

fn excess(kind: Outlier, value: f32, reference: f32) -> f32 {
    match kind {
        Outlier::Hot => value - reference,
        Outlier::Dead => reference - value,
    }
}

fn exceeds(kind: Outlier, value: f32, reference: f32) -> bool {
    let beyond = match kind {
        Outlier::Hot => value > OUTLIER_RATIO * reference,
        Outlier::Dead => value * OUTLIER_RATIO < reference,
    };
    beyond && excess(kind, value, reference) > OUTLIER_FLOOR
}

fn classify(rings: &Rings, mosaic: &Mosaic, x: usize, y: usize) -> Option<Outlier> {
    let value = mosaic.get(x, y);
    let [fx, fy] = rings.sites(mosaic, x, y).next()?;
    let first = mosaic.get(fx, fy);
    let kind = [Outlier::Hot, Outlier::Dead]
        .into_iter()
        .find(|kind| exceeds(*kind, value, first))?;
    let mut ring = rings.values(mosaic, x, y, |_| false);
    if ring.len < MIN_RING {
        return None;
    }
    let reference = match kind {
        Outlier::Hot => ring.slice().iter().copied().fold(f32::MIN, f32::max),
        Outlier::Dead => ring.slice().iter().copied().fold(f32::MAX, f32::min),
    };
    if !exceeds(kind, value, reference) {
        return None;
    }
    let threshold = LIGHT_SPREAD * excess(kind, value, ring.median()?);
    let spill: Vec<f32> = [[1, 0], [-1, 0], [0, 1], [0, -1]]
        .into_iter()
        .filter_map(|[dx, dy]| Some([x.checked_add_signed(dx)?, y.checked_add_signed(dy)?]))
        .filter(|&[ax, ay]| ax < mosaic.width && ay < mosaic.height)
        .filter_map(|[ax, ay]| {
            let mut ring = rings.values(mosaic, ax, ay, |_| false);
            ring.median().map(|m| excess(kind, mosaic.get(ax, ay), m))
        })
        .collect();
    let mean_spill = spill.iter().sum::<f32>() / spill.len().max(1) as f32;
    (mean_spill <= threshold).then_some(kind)
}

fn detect(rings: &Rings, mosaic: &Mosaic) -> Vec<[usize; 2]> {
    (0..mosaic.height)
        .into_par_iter()
        .flat_map_iter(|y| {
            (0..mosaic.width)
                .filter(move |&x| classify(rings, mosaic, x, y).is_some())
                .map(move |x| [x, y])
        })
        .collect()
}

fn site_order(site: &[usize; 2]) -> (usize, usize) {
    (site[1], site[0])
}

pub(crate) fn repair(mosaic: &mut Mosaic, flagged: &[[usize; 2]]) {
    if mosaic.width == 0 || mosaic.height == 0 {
        return;
    }
    let rings = Rings::new(&mosaic.cfa);
    let mut bad = detect(&rings, mosaic);
    bad.extend(
        flagged
            .iter()
            .filter(|[x, y]| *x < mosaic.width && *y < mosaic.height),
    );
    bad.sort_unstable_by_key(site_order);
    bad.dedup();
    let is_bad = |site: [usize; 2]| {
        bad.binary_search_by_key(&site_order(&site), site_order)
            .is_ok()
    };
    let fixes: Vec<Option<f32>> = bad
        .par_iter()
        .map(|&[x, y]| rings.values(mosaic, x, y, is_bad).median())
        .collect();
    bad.iter()
        .zip(fixes)
        .filter_map(|(site, fix)| fix.map(|v| (site, v)))
        .for_each(|(&[x, y], v)| mosaic.set(x, y, v));
}

#[cfg(test)]
mod tests;
