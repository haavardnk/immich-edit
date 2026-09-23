use crate::histogram::{LUMA_SCALE, display_luma, sample_step, weighted_luma};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const LEVELS: usize = 256;
pub const WAVEFORM_COLUMNS: usize = 512;
pub const PARADE_COLUMNS: usize = 192;
pub const VECTORSCOPE_SIZE: usize = 384;

pub(crate) const WAVEFORM_CELLS: usize = WAVEFORM_COLUMNS * LEVELS;
pub(crate) const PARADE_CELLS: usize = PARADE_COLUMNS * LEVELS * 3;
pub(crate) const VECTORSCOPE_CELLS: usize = VECTORSCOPE_SIZE * VECTORSCOPE_SIZE;
pub(crate) const SCOPE_CELLS: usize = WAVEFORM_CELLS + PARADE_CELLS + VECTORSCOPE_CELLS;

const CB_DENOM: i32 = 255 * 18_556;
const CR_DENOM: i32 = 255 * 15_748;
const DENSITY_GAMMA: f32 = 1.0 / 2.2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeGrid {
    pub width: u16,
    pub height: u16,
    pub channels: u8,
    pub max_count: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeGrids {
    pub waveform: ScopeGrid,
    pub parade: ScopeGrid,
    pub vectorscope: ScopeGrid,
}

impl ScopeGrids {
    pub fn from_rgb_u8(pixels: &[u8], width: usize, height: usize) -> Self {
        let counts = accumulate(pixels, width, height);
        encode(&counts.waveform, &counts.parade, &counts.vectorscope)
    }

    pub(crate) fn from_counts(counts: &[u32]) -> Self {
        let (waveform, rest) = counts.split_at(WAVEFORM_CELLS);
        let (parade, vectorscope) = rest.split_at(PARADE_CELLS);
        encode(waveform, parade, &vectorscope[..VECTORSCOPE_CELLS])
    }
}

pub(crate) fn row_step(width: usize, rows: usize) -> usize {
    sample_step(width * rows)
}

fn accumulate(pixels: &[u8], width: usize, height: usize) -> Counts {
    let rows = (pixels.len() / 3 / width.max(1)).min(height);
    if width == 0 || rows == 0 {
        return Counts::zero();
    }
    let step = row_step(width, rows);
    let band = rows.div_ceil(rayon::current_num_threads().max(1)).max(1);
    let starts: Vec<usize> = (0..rows).step_by(band).collect();
    starts
        .into_par_iter()
        .map(|start| {
            let mut counts = Counts::zero();
            counts.add_band(pixels, width, start, (start + band).min(rows), step);
            counts
        })
        .reduce(Counts::zero, Counts::merge)
}

struct Counts {
    waveform: Vec<u32>,
    parade: Vec<u32>,
    vectorscope: Vec<u32>,
}

impl Counts {
    fn zero() -> Self {
        Self {
            waveform: vec![0; WAVEFORM_CELLS],
            parade: vec![0; PARADE_CELLS],
            vectorscope: vec![0; VECTORSCOPE_CELLS],
        }
    }

    fn merge(mut self, other: Self) -> Self {
        add_into(&mut self.waveform, &other.waveform);
        add_into(&mut self.parade, &other.parade);
        add_into(&mut self.vectorscope, &other.vectorscope);
        self
    }

    fn add_band(&mut self, pixels: &[u8], width: usize, start: usize, end: usize, step: usize) {
        let stride = width * 3;
        for y in (start..end).filter(|y| y % step == 0) {
            self.add_row(&pixels[y * stride..(y + 1) * stride], width);
        }
    }

    fn add_row(&mut self, row: &[u8], width: usize) {
        for (x, px) in row.chunks_exact(3).enumerate() {
            self.add_pixel(x, width, px[0], px[1], px[2]);
        }
    }

    fn add_pixel(&mut self, x: usize, width: usize, r: u8, g: u8, b: u8) {
        let level = display_luma(r, g, b);
        self.waveform[(LEVELS - 1 - level) * WAVEFORM_COLUMNS + x * WAVEFORM_COLUMNS / width] += 1;

        let column = x * PARADE_COLUMNS / width;
        let cell = |v: u8| ((LEVELS - 1 - v as usize) * PARADE_COLUMNS + column) * 3;
        self.parade[cell(r)] += 1;
        self.parade[cell(g) + 1] += 1;
        self.parade[cell(b) + 2] += 1;

        let (vx, vy) = vectorscope_cell(r, g, b);
        self.vectorscope[vy * VECTORSCOPE_SIZE + vx] += 1;
    }
}

fn add_into(target: &mut [u32], source: &[u32]) {
    target
        .iter_mut()
        .zip(source)
        .for_each(|(a, b)| *a = a.saturating_add(*b));
}

/// BT.709 Y'CbCr plotted as x = Cb, y = Cr with +Cr upward, matching a broadcast vectorscope.
/// Exact integer arithmetic, so `scopes.wgsl` lands every pixel in the same cell.
fn vectorscope_cell(r: u8, g: u8, b: u8) -> (usize, usize) {
    let y = weighted_luma(r, g, b) as i32;
    let scale = LUMA_SCALE as i32;
    let size = VECTORSCOPE_SIZE as i32;
    let half = size / 2;
    let x = (size * (scale * b as i32 - y) + half * CB_DENOM) / CB_DENOM;
    let v = (half * CR_DENOM - size * (scale * r as i32 - y)) / CR_DENOM;
    (x.clamp(0, size - 1) as usize, v.clamp(0, size - 1) as usize)
}

fn encode(waveform: &[u32], parade: &[u32], vectorscope: &[u32]) -> ScopeGrids {
    ScopeGrids {
        waveform: encode_grid(waveform, WAVEFORM_COLUMNS, LEVELS, 1),
        parade: encode_grid(parade, PARADE_COLUMNS, LEVELS, 3),
        vectorscope: encode_grid(vectorscope, VECTORSCOPE_SIZE, VECTORSCOPE_SIZE, 1),
    }
}

fn encode_grid(counts: &[u32], width: usize, height: usize, channels: usize) -> ScopeGrid {
    let max_count = counts.iter().copied().max().unwrap_or(0);
    let data = match max_count {
        0 => vec![0u8; counts.len()],
        max => {
            let inv = 1.0 / max as f32;
            counts
                .par_iter()
                .map(|&c| ((c as f32 * inv).powf(DENSITY_GAMMA) * 255.0).round() as u8)
                .collect()
        }
    };
    ScopeGrid {
        width: width as u16,
        height: height as u16,
        channels: channels as u8,
        max_count,
        data,
    }
}

#[cfg(test)]
mod tests;
