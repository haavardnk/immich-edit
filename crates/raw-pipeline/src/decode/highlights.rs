mod field;

use crate::sensor_sample::{camera_wb_coeffs, cfa_block};
use field::Field;
use rayon::prelude::*;

const CLIP: f32 = 0.987;
const PLATEAU: f32 = 0.998;
const MIN_PLATEAU: usize = 64;
const SATURATION_FLOOR: f32 = 0.5;
const CHROMA_FLOOR: f32 = 0.2;
const MIN_SAMPLES: usize = 100;
const BLOCK: usize = 3;
const DILATE: usize = 3;
const PASSES: usize = 4;
const OTHERS: [(usize, usize); 3] = [(1, 2), (0, 2), (0, 1)];

struct Grid {
    width: usize,
    bits: Vec<u8>,
}

impl Grid {
    fn has(&self, x: usize, y: usize, c: usize) -> bool {
        self.bits[(y / BLOCK) * self.width + x / BLOCK] & (1 << c) != 0
    }

    fn near(&self, x: usize, y: usize) -> bool {
        self.bits[(y / BLOCK) * self.width + x / BLOCK] != 0
    }
}

#[derive(Default, Clone, Copy)]
struct Chroma {
    sum: [f64; 3],
    count: [usize; 3],
}

impl Chroma {
    fn add(mut self, c: usize, delta: f32) -> Self {
        self.sum[c] += f64::from(delta);
        self.count[c] += 1;
        self
    }

    fn merge(mut self, other: Self) -> Self {
        for c in 0..3 {
            self.sum[c] += other.sum[c];
            self.count[c] += other.count[c];
        }
        self
    }

    fn offsets(self) -> [f32; 3] {
        std::array::from_fn(|c| {
            if self.count[c] > MIN_SAMPLES {
                (self.sum[c] / self.count[c] as f64) as f32
            } else {
                0.0
            }
        })
    }
}

struct Sensor {
    wb: [f32; 3],
    clip: [f32; 3],
}

impl Sensor {
    fn new<I: ParallelIterator<Item = (usize, f32)>>(
        wb_coeffs: [f32; 4],
        samples: impl Fn() -> I,
    ) -> Option<Self> {
        let wb = camera_wb_coeffs(wb_coeffs);
        if !wb.iter().all(|w| w.is_finite() && *w > 0.0) {
            return None;
        }
        let max = samples()
            .fold(
                || [0.0f32; 3],
                |mut m, (c, v)| {
                    m[c] = m[c].max(v);
                    m
                },
            )
            .reduce(|| [0.0; 3], |a, b| std::array::from_fn(|c| a[c].max(b[c])));
        let count = samples()
            .fold(
                || [0usize; 3],
                |mut n, (c, v)| {
                    n[c] += usize::from(v >= max[c] * PLATEAU);
                    n
                },
            )
            .reduce(|| [0; 3], |a, b| std::array::from_fn(|c| a[c] + b[c]));
        let plateau: [Option<f32>; 3] = std::array::from_fn(|c| {
            (count[c] >= MIN_PLATEAU && max[c] >= SATURATION_FLOOR).then_some(max[c])
        });
        let shared = plateau
            .iter()
            .flatten()
            .copied()
            .reduce(f32::max)
            .unwrap_or(1.0);
        let clip = std::array::from_fn(|c| CLIP * plateau[c].unwrap_or(shared));
        Some(Self { wb, clip })
    }

    fn clipped(&self, c: usize, v: f32) -> bool {
        v >= self.clip[c]
    }
}

fn blend(level: [f32; 3], c: usize) -> f32 {
    let (a, b) = OTHERS[c];
    let mean = 0.5 * (level[a].cbrt() + level[b].cbrt());
    mean * mean * mean
}

fn desaturation(ratio: [f32; 3], j: usize, k: usize, known: f32, clip: f32) -> f32 {
    let need = clip * ratio[j] - ratio[k] * known;
    if need <= 0.0 {
        return 0.0;
    }
    let slope = (1.0 - ratio[k]) * known - clip * (1.0 - ratio[j]);
    if slope <= 0.0 {
        return 0.0;
    }
    (need / slope).min(1.0)
}

#[derive(Clone, Copy)]
struct Window {
    mean: [f32; 3],
    peak: [f32; 3],
}

impl Window {
    fn saturated(&self, sensor: &Sensor, k: usize) -> bool {
        sensor.clipped(k, self.peak[k])
    }

    fn count(&self, sensor: &Sensor) -> usize {
        (0..3).filter(|k| self.saturated(sensor, *k)).count()
    }

    fn multi(&self, sensor: &Sensor) -> bool {
        self.count(sensor) >= 2
    }

    fn levels(&self, sensor: &Sensor) -> [f32; 3] {
        let mut level: [f32; 3] = std::array::from_fn(|k| (sensor.wb[k] * self.mean[k]).max(0.0));
        for k in (0..PASSES)
            .flat_map(|_| 0..3)
            .filter(|k| self.saturated(sensor, *k))
        {
            level[k] = level[k].max(blend(level, k));
        }
        level
    }

    fn opposed(&self, sensor: &Sensor, c: usize) -> f32 {
        blend(self.levels(sensor), c)
    }

    fn reconstruct(&self, sensor: &Sensor, chroma: [f32; 3], c: usize, v: f32) -> f32 {
        (self.opposed(sensor, c) + chroma[c]).max(v * sensor.wb[c]) / sensor.wb[c]
    }

    fn field_sample(&self, sensor: &Sensor, chroma: [f32; 3]) -> Option<[f32; 3]> {
        if self.multi(sensor) {
            return None;
        }
        let level = self.levels(sensor);
        Some(std::array::from_fn(|k| {
            if self.saturated(sensor, k) {
                level[k].max(blend(level, k) + chroma[k])
            } else {
                level[k]
            }
        }))
    }

    fn clipped_levels(&self, sensor: &Sensor) -> [f32; 3] {
        std::array::from_fn(|k| {
            if self.saturated(sensor, k) {
                self.peak[k] * sensor.wb[k]
            } else {
                0.0
            }
        })
    }

    fn fill(&self, sensor: &Sensor, ratio: [f32; 3], c: usize, v: f32) -> f32 {
        let clipped = self.clipped_levels(sensor);
        let known = (0..3)
            .find(|k| !self.saturated(sensor, *k))
            .map(|j| (j, (self.mean[j] * sensor.wb[j]).max(0.0)));
        let t = known.map_or(0.0f32, |(j, level)| {
            (0..3).filter(|k| *k != j).fold(0.0f32, |t, k| {
                t.max(desaturation(ratio, j, k, level, clipped[k]))
            })
        });
        let hue = ratio.map(|r| r + (1.0 - r) * t);
        let base = known.map_or(0.0, |(j, level)| level / hue[j]);
        let level = (0..3).fold(base, |m, k| m.max(clipped[k] / hue[k]));
        (level * hue[c] / sensor.wb[c]).max(v)
    }

    fn chroma_sample(&self, sensor: &Sensor, c: usize, v: f32) -> Option<f32> {
        if v <= CHROMA_FLOOR || (0..3).any(|k| self.saturated(sensor, k)) {
            return None;
        }
        Some(v * sensor.wb[c] - self.opposed(sensor, c))
    }
}

fn clipped_grid(
    width: usize,
    height: usize,
    bits: impl Fn(usize, usize) -> u8 + Sync,
) -> Option<Grid> {
    let gw = width.div_ceil(BLOCK);
    let gh = height.div_ceil(BLOCK);
    let mut grid = vec![0u8; gw * gh];
    grid.par_chunks_mut(gw.max(1))
        .enumerate()
        .for_each(|(by, row)| {
            let ys = by * BLOCK..((by + 1) * BLOCK).min(height);
            for (y, x) in ys.flat_map(|y| (0..width).map(move |x| (y, x))) {
                row[x / BLOCK] |= bits(x, y);
            }
        });
    if grid.iter().all(|b| *b == 0) {
        return None;
    }
    let horizontal: Vec<u8> = (0..gw * gh)
        .map(|i| {
            let (y, x) = (i / gw, i % gw);
            let lo = x.saturating_sub(DILATE);
            let hi = (x + DILATE).min(gw - 1);
            grid[y * gw + lo..=y * gw + hi].iter().fold(0, |a, b| a | b)
        })
        .collect();
    let bits = (0..gw * gh)
        .map(|i| {
            let (y, x) = (i / gw, i % gw);
            let lo = y.saturating_sub(DILATE);
            let hi = (y + DILATE).min(gh - 1);
            (lo..=hi).fold(0, |a, yy| a | horizontal[yy * gw + x])
        })
        .collect();
    Some(Grid { width: gw, bits })
}

pub(super) fn reconstruct_mosaic(
    data: &mut [f32],
    width: usize,
    height: usize,
    pattern: &str,
    wb_coeffs: [f32; 4],
) {
    if width == 0 || height == 0 {
        return;
    }
    let (dim, table) = cfa_block(pattern);
    let color = |x: usize, y: usize| table[(y % dim) * dim + x % dim];
    let src: &[f32] = data;
    let Some(sensor) = Sensor::new(wb_coeffs, || {
        src.par_chunks_exact(width)
            .enumerate()
            .flat_map_iter(move |(y, row)| {
                row.iter().enumerate().map(move |(x, v)| (color(x, y), *v))
            })
    }) else {
        return;
    };
    let Some(grid) = clipped_grid(width, height, |x, y| {
        let c = color(x, y);
        u8::from(sensor.clipped(c, src[y * width + x])) << c
    }) else {
        return;
    };
    let window = |x: usize, y: usize| mosaic_window(src, width, height, color, x, y);
    let chroma = (0..height)
        .into_par_iter()
        .fold(Chroma::default, |acc, y| {
            (0..width).fold(acc, |acc, x| {
                let c = color(x, y);
                if !grid.has(x, y, c) {
                    return acc;
                }
                match window(x, y).chroma_sample(&sensor, c, src[y * width + x]) {
                    Some(delta) => acc.add(c, delta),
                    None => acc,
                }
            })
        })
        .reduce(Chroma::default, Chroma::merge)
        .offsets();
    let field = Field::new(
        width,
        height,
        |x, y| {
            if grid.near(x, y) {
                window(x, y).count(&sensor)
            } else {
                0
            }
        },
        |x, y| window(x, y).field_sample(&sensor, chroma),
    );
    let rows: Vec<Option<Vec<f32>>> = (0..height)
        .into_par_iter()
        .map(|y| {
            let row = &src[y * width..(y + 1) * width];
            if !row
                .iter()
                .enumerate()
                .any(|(x, v)| sensor.clipped(color(x, y), *v))
            {
                return None;
            }
            let out = row.iter().enumerate().map(|(x, &v)| {
                let c = color(x, y);
                if !sensor.clipped(c, v) {
                    return v;
                }
                let w = window(x, y);
                if w.multi(&sensor) {
                    return w.fill(&sensor, field.ratio(x, y, w.clipped_levels(&sensor)), c, v);
                }
                w.reconstruct(&sensor, chroma, c, v)
            });
            Some(out.collect())
        })
        .collect();
    data.par_chunks_exact_mut(width)
        .zip(rows)
        .for_each(|(dst, row)| {
            if let Some(row) = row {
                dst.copy_from_slice(&row);
            }
        });
}

fn mosaic_window(
    src: &[f32],
    width: usize,
    height: usize,
    color: impl Fn(usize, usize) -> usize,
    x: usize,
    y: usize,
) -> Window {
    let mut sum = [0.0f32; 3];
    let mut count = [0u32; 3];
    let mut peak = [0.0f32; 3];
    let ys = y.saturating_sub(1)..=(y + 1).min(height - 1);
    let xs = x.saturating_sub(1)..=(x + 1).min(width - 1);
    for (yy, xx) in ys.flat_map(|yy| xs.clone().map(move |xx| (yy, xx))) {
        let k = color(xx, yy);
        let v = src[yy * width + xx];
        sum[k] += v;
        count[k] += 1;
        peak[k] = peak[k].max(v);
    }
    let mean = std::array::from_fn(|k| {
        if count[k] == 0 {
            0.0
        } else {
            sum[k] / count[k] as f32
        }
    });
    Window { mean, peak }
}

pub(super) fn reconstruct_rgb(data: &mut [f32], width: usize, height: usize, wb_coeffs: [f32; 4]) {
    if width == 0 || height == 0 {
        return;
    }
    let src: &[f32] = data;
    let Some(sensor) = Sensor::new(wb_coeffs, || {
        src.par_iter().enumerate().map(|(i, v)| (i % 3, *v))
    }) else {
        return;
    };
    let Some(grid) = clipped_grid(width, height, |x, y| {
        let i = (y * width + x) * 3;
        (0..3).fold(0, |bits, c| {
            bits | (u8::from(sensor.clipped(c, src[i + c])) << c)
        })
    }) else {
        return;
    };
    let window = |px: &[f32]| -> Window {
        let mean = [px[0], px[1], px[2]];
        Window { mean, peak: mean }
    };
    let chroma = src
        .par_chunks_exact(3)
        .enumerate()
        .fold(Chroma::default, |acc, (i, px)| {
            let (x, y) = (i % width, i / width);
            (0..3).fold(acc, |acc, c| {
                if !grid.has(x, y, c) {
                    return acc;
                }
                match window(px).chroma_sample(&sensor, c, px[c]) {
                    Some(delta) => acc.add(c, delta),
                    None => acc,
                }
            })
        })
        .reduce(Chroma::default, Chroma::merge)
        .offsets();
    let pixel = |x: usize, y: usize| window(&src[(y * width + x) * 3..][..3]);
    let field = Field::new(
        width,
        height,
        |x, y| {
            if grid.near(x, y) {
                pixel(x, y).count(&sensor)
            } else {
                0
            }
        },
        |x, y| pixel(x, y).field_sample(&sensor, chroma),
    );
    data.par_chunks_exact_mut(3)
        .enumerate()
        .for_each(|(i, px)| {
            let w = window(px);
            let ratio = w
                .multi(&sensor)
                .then(|| field.ratio(i % width, i / width, w.clipped_levels(&sensor)));
            for (c, v) in px.iter_mut().enumerate() {
                if !sensor.clipped(c, *v) {
                    continue;
                }
                *v = match ratio {
                    Some(ratio) => w.fill(&sensor, ratio, c, *v),
                    None => w.reconstruct(&sensor, chroma, c, *v),
                };
            }
        });
}

#[cfg(test)]
mod tests;
