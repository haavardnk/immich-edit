use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

pub const LEVELS: usize = 256;
pub const WAVEFORM_COLUMNS: usize = 512;
pub const PARADE_COLUMNS: usize = 192;
pub const VECTORSCOPE_SIZE: usize = 384;

const CB_SCALE: f32 = 1.8556;
const CR_SCALE: f32 = 1.5748;
const DENSITY_GAMMA: f32 = 1.0 / 2.2;
const SUBSAMPLE_ABOVE: usize = 500_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeGrid {
    pub width: u16,
    pub height: u16,
    pub channels: u8,
    pub max_count: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeGrids {
    pub waveform: ScopeGrid,
    pub parade: ScopeGrid,
    pub vectorscope: ScopeGrid,
}

impl ScopeGrids {
    pub fn from_rgb_u8(pixels: &[u8], width: usize, height: usize) -> Self {
        accumulate(pixels, 3, width, height).encode()
    }

    pub fn from_rgba8(pixels: &[u8], width: usize, height: usize) -> Self {
        accumulate(pixels, 4, width, height).encode()
    }
}

fn accumulate(pixels: &[u8], comps: usize, width: usize, height: usize) -> Counts {
    let rows = (pixels.len() / comps / width.max(1)).min(height);
    if width == 0 || rows == 0 {
        return Counts::zero();
    }
    let step = if width * rows > SUBSAMPLE_ABOVE { 2 } else { 1 };
    let band = rows.div_ceil(rayon::current_num_threads().max(1)).max(1);
    let starts: Vec<usize> = (0..rows).step_by(band).collect();
    starts
        .into_par_iter()
        .map(|start| {
            let mut counts = Counts::zero();
            counts.add_band(pixels, comps, width, start, (start + band).min(rows), step);
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
            waveform: vec![0; WAVEFORM_COLUMNS * LEVELS],
            parade: vec![0; PARADE_COLUMNS * LEVELS * 3],
            vectorscope: vec![0; VECTORSCOPE_SIZE * VECTORSCOPE_SIZE],
        }
    }

    fn merge(mut self, other: Self) -> Self {
        add_into(&mut self.waveform, &other.waveform);
        add_into(&mut self.parade, &other.parade);
        add_into(&mut self.vectorscope, &other.vectorscope);
        self
    }

    fn add_band(
        &mut self,
        pixels: &[u8],
        comps: usize,
        width: usize,
        start: usize,
        end: usize,
        step: usize,
    ) {
        let stride = width * comps;
        for y in (start..end).filter(|y| y % step == 0) {
            self.add_row(&pixels[y * stride..(y + 1) * stride], comps, width);
        }
    }

    fn add_row(&mut self, row: &[u8], comps: usize, width: usize) {
        for (x, px) in row.chunks_exact(comps).enumerate() {
            self.add_pixel(x, width, px[0], px[1], px[2]);
        }
    }

    fn add_pixel(&mut self, x: usize, width: usize, r: u8, g: u8, b: u8) {
        let level = luma_level(r, g, b);
        self.waveform[(LEVELS - 1 - level) * WAVEFORM_COLUMNS + x * WAVEFORM_COLUMNS / width] += 1;

        let column = x * PARADE_COLUMNS / width;
        let cell = |v: u8| ((LEVELS - 1 - v as usize) * PARADE_COLUMNS + column) * 3;
        self.parade[cell(r)] += 1;
        self.parade[cell(g) + 1] += 1;
        self.parade[cell(b) + 2] += 1;

        let (vx, vy) = vectorscope_cell(r, g, b);
        self.vectorscope[vy * VECTORSCOPE_SIZE + vx] += 1;
    }

    fn encode(self) -> ScopeGrids {
        ScopeGrids {
            waveform: encode_grid(&self.waveform, WAVEFORM_COLUMNS, LEVELS, 1),
            parade: encode_grid(&self.parade, PARADE_COLUMNS, LEVELS, 3),
            vectorscope: encode_grid(&self.vectorscope, VECTORSCOPE_SIZE, VECTORSCOPE_SIZE, 1),
        }
    }
}

fn add_into(target: &mut [u32], source: &[u32]) {
    target
        .iter_mut()
        .zip(source)
        .for_each(|(a, b)| *a = a.saturating_add(*b));
}

fn luma_level(r: u8, g: u8, b: u8) -> usize {
    let value = LUMA_R * r as f32 + LUMA_G * g as f32 + LUMA_B * b as f32;
    (value as usize).min(LEVELS - 1)
}

/// BT.709 Y'CbCr plotted as x = Cb, y = Cr with +Cr upward, matching a broadcast vectorscope.
fn vectorscope_cell(r: u8, g: u8, b: u8) -> (usize, usize) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let y = LUMA_R * rf + LUMA_G * gf + LUMA_B * bf;
    let cb = (bf - y) / CB_SCALE;
    let cr = (rf - y) / CR_SCALE;
    let last = VECTORSCOPE_SIZE - 1;
    let x = ((cb + 0.5) * VECTORSCOPE_SIZE as f32) as usize;
    let v = ((0.5 - cr) * VECTORSCOPE_SIZE as f32) as usize;
    (x.min(last), v.min(last))
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
