use rayon::prelude::*;

const CELL: usize = 8;
const BRIGHT: f32 = 0.5;
const TAU: f32 = 0.05;
const RADII: [usize; 7] = [64, 32, 16, 8, 4, 2, 1];
const BOX_PASSES: usize = 3;
const RATIO_FLOOR: f32 = 0.5;
const KEEP_EXCESS: f32 = 1.15;
const NEUTRAL_EXCESS: f32 = 2.0;
const UNIFORM_SPAN: f32 = 32.0;
const CORE_RADIUS: usize = 1;

type Cell = [f32; 4];

fn scan<T: Send>(
    width: usize,
    height: usize,
    init: impl Fn() -> T + Sync,
    visit: impl Fn(&mut [T], usize, usize) + Sync,
) -> Vec<T> {
    let cw = width.div_ceil(CELL);
    (0..height.div_ceil(CELL))
        .into_par_iter()
        .flat_map_iter(|cy| {
            let mut row: Vec<T> = (0..cw).map(|_| init()).collect();
            let ys = cy * CELL..((cy + 1) * CELL).min(height);
            for (y, x) in ys.flat_map(|y| (0..width).map(move |x| (y, x))) {
                visit(&mut row, x, y);
            }
            row
        })
        .collect()
}

fn dilate(mask: &[bool], width: usize, height: usize) -> Vec<bool> {
    (0..width * height)
        .map(|i| {
            let (x, y) = (i % width, i / width);
            let xs = x.saturating_sub(1)..=(x + 1).min(width - 1);
            (y.saturating_sub(1)..=(y + 1).min(height - 1))
                .any(|yy| xs.clone().any(|xx| mask[yy * width + xx]))
        })
        .collect()
}

fn box_rows(src: &[Cell], width: usize, radius: usize) -> Vec<Cell> {
    let mut out = vec![[0.0; 4]; src.len()];
    out.par_chunks_mut(width)
        .zip(src.par_chunks(width))
        .for_each(|(dst, row)| {
            let last = row.len() as isize - 1;
            let at = |i: isize| row[i.clamp(0, last) as usize];
            let r = radius as isize;
            let mut sum = (-r..=r).fold([0.0f64; 4], |s, i| {
                std::array::from_fn(|k| s[k] + f64::from(at(i)[k]))
            });
            let scale = 1.0 / (2 * radius + 1) as f64;
            for (i, d) in dst.iter_mut().enumerate() {
                *d = sum.map(|v| (v * scale) as f32);
                let add = at(i as isize + r + 1);
                let sub = at(i as isize - r);
                sum = std::array::from_fn(|k| sum[k] + f64::from(add[k]) - f64::from(sub[k]));
            }
        });
    out
}

fn transpose(src: &[Cell], width: usize, height: usize) -> Vec<Cell> {
    (0..width * height)
        .map(|i| src[(i % height) * width + i / height])
        .collect()
}

fn blur(cells: &[Cell], width: usize, height: usize, radius: usize) -> Vec<Cell> {
    (0..BOX_PASSES).fold(cells.to_vec(), |acc, _| {
        let rows = box_rows(&acc, width, radius);
        let cols = box_rows(&transpose(&rows, width, height), height, radius);
        transpose(&cols, height, width)
    })
}

fn spread(cells: &[Cell], width: usize, height: usize, color: &mut [[f32; 3]]) -> bool {
    let max_weight = cells.iter().fold(0.0f32, |m, c| m.max(c[0]));
    if max_weight <= 0.0 {
        return false;
    }
    for radius in RADII {
        let blurred = blur(cells, width, height, radius);
        for (dst, b) in color.iter_mut().zip(&blurred).filter(|(_, b)| b[0] > 0.0) {
            let confidence = (b[0] / (TAU * max_weight)).min(1.0);
            for (k, v) in dst.iter_mut().enumerate() {
                *v += confidence * (b[k + 1] / b[0] - *v);
            }
        }
    }
    true
}

fn fade(value: f32, keep: f32, neutral: f32) -> f32 {
    ((value - keep) / (neutral - keep)).clamp(0.0, 1.0)
}

fn segments(hit: &[bool], width: usize, height: usize) -> (Vec<usize>, usize) {
    let mut label = vec![usize::MAX; hit.len()];
    let mut count = 0;
    let mut stack = Vec::new();
    for start in (0..hit.len()).filter(|i| hit[*i]) {
        if label[start] != usize::MAX {
            continue;
        }
        label[start] = count;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % width, i / width);
            let xs = x.saturating_sub(1)..=(x + 1).min(width - 1);
            let ys = y.saturating_sub(1)..=(y + 1).min(height - 1);
            for j in ys.flat_map(|yy| xs.clone().map(move |xx| yy * width + xx)) {
                if hit[j] && label[j] == usize::MAX {
                    label[j] = count;
                    stack.push(j);
                }
            }
        }
        count += 1;
    }
    (label, count)
}

fn uniform(color: &mut [[f32; 3]], cells: &[Cell], hit: &[bool], width: usize, height: usize) {
    let (label, count) = segments(hit, width, height);
    let mut totals = vec![[0.0f64; 5]; count];
    for (i, l) in label.iter().enumerate().filter(|(_, l)| **l != usize::MAX) {
        totals[*l][4] += 1.0;
        let (x, y) = (i % width, i / width);
        let xs = x.saturating_sub(1)..=(x + 1).min(width - 1);
        let ys = y.saturating_sub(1)..=(y + 1).min(height - 1);
        for j in ys.flat_map(|yy| xs.clone().map(move |xx| yy * width + xx)) {
            if hit[j] {
                continue;
            }
            for (dst, v) in totals[*l][..4].iter_mut().zip(cells[j]) {
                *dst += f64::from(v);
            }
        }
    }
    for (dst, l) in color
        .iter_mut()
        .zip(&label)
        .filter(|(_, l)| **l != usize::MAX)
    {
        let [weight, r, g, b, size] = totals[*l];
        if weight <= 0.0 {
            continue;
        }
        let keep = ((size.sqrt() as f32 - UNIFORM_SPAN) / UNIFORM_SPAN).clamp(0.0, 1.0);
        let mean = [r, g, b].map(|v| (v / weight) as f32);
        *dst = std::array::from_fn(|k| mean[k] + (dst[k] - mean[k]) * keep);
    }
}

pub(super) struct Field {
    width: usize,
    height: usize,
    color: Vec<[f32; 4]>,
}

impl Field {
    pub(super) fn new(
        width: usize,
        height: usize,
        clipped: impl Fn(usize, usize) -> usize + Sync,
        sample: impl Fn(usize, usize) -> Option<[f32; 3]> + Sync,
    ) -> Self {
        let cw = width.div_ceil(CELL);
        let ch = height.div_ceil(CELL);
        let counts = scan(
            width,
            height,
            || (false, 0.0f32, 0.0f32),
            |row, x, y| {
                let n = clipped(x, y);
                let cell = &mut row[x / CELL];
                cell.0 |= n >= 2;
                cell.1 += f32::from(u8::from(n == 3));
                cell.2 += 1.0;
            },
        );
        let hit: Vec<bool> = counts.iter().map(|c| c.0).collect();
        if !hit.contains(&true) {
            return Self {
                width: cw,
                height: ch,
                color: vec![[1.0, 1.0, 1.0, 0.0]; cw * ch],
            };
        }
        let band = dilate(&hit, cw, ch);
        let cells = scan(
            width,
            height,
            || [0.0f32; 4],
            |row, x, y| {
                if !band[(y / CELL) * cw + x / CELL] {
                    return;
                }
                let Some(level) = sample(x, y) else {
                    return;
                };
                let peak = level.iter().fold(0.0f32, |m, v| m.max(*v));
                if peak <= BRIGHT || !peak.is_finite() {
                    return;
                }
                let weight = peak * peak;
                let cell = &mut row[x / CELL];
                cell[0] += weight;
                for (dst, v) in cell[1..].iter_mut().zip(level) {
                    *dst += weight * v;
                }
            },
        );
        let mut color = vec![[1.0; 3]; cw * ch];
        if spread(&cells, cw, ch, &mut color) {
            uniform(&mut color, &cells, &hit, cw, ch);
        }
        let full: Vec<Cell> = counts
            .iter()
            .map(|c| [c.1 / c.2.max(1.0), 0.0, 0.0, 0.0])
            .collect();
        let core = blur(&full, cw, ch, CORE_RADIUS);
        Self {
            width: cw,
            height: ch,
            color: color
                .iter()
                .zip(&core)
                .map(|(c, k)| [c[0], c[1], c[2], k[0].min(1.0)])
                .collect(),
        }
    }

    fn lerp(&self, x: usize, y: usize) -> [f32; 4] {
        let fx = ((x as f32 + 0.5) / CELL as f32 - 0.5).clamp(0.0, (self.width - 1) as f32);
        let fy = ((y as f32 + 0.5) / CELL as f32 - 0.5).clamp(0.0, (self.height - 1) as f32);
        let x0 = fx as usize;
        let y0 = fy as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;
        let at = |xx: usize, yy: usize| self.color[yy * self.width + xx];
        std::array::from_fn(|c| {
            let top = at(x0, y0)[c] + (at(x1, y0)[c] - at(x0, y0)[c]) * tx;
            let bottom = at(x0, y1)[c] + (at(x1, y1)[c] - at(x0, y1)[c]) * tx;
            top + (bottom - top) * ty
        })
    }

    pub(super) fn ratio(&self, x: usize, y: usize, clipped: [f32; 3]) -> [f32; 3] {
        let [r, g, b, core] = self.lerp(x, y);
        let rgb = [r, g, b];
        let peak = rgb.iter().fold(0.0f32, |m, v| m.max(*v));
        if !peak.is_finite() || peak <= 0.0 {
            return [1.0; 3];
        }
        let excess = (0..3).fold(0.0f32, |m, k| m.max(clipped[k] / rgb[k].max(f32::EPSILON)));
        let neutral = fade(excess, KEEP_EXCESS, NEUTRAL_EXCESS).max(core);
        rgb.map(|v| {
            let r = (v / peak).max(RATIO_FLOOR);
            r + (1.0 - r) * neutral
        })
    }
}
