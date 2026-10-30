use std::iter::once;

use super::Rect;
use crate::cpu::demosaic::parse_xtrans;
use crate::cpu::masked::SelectorTaps;
use crate::gpu::passes::sharpen::SHARPEN_KERNEL_HALF;
use crate::ops::capture_sharpen::ITERATIONS;
use crate::ops::denoise::{CHROMA_LEVELS, LUMA_LEVELS};

pub const HIST_ALIGN: u32 = 4;
const BAYER_PERIOD: u32 = 2;
const XTRANS_PERIOD: u32 = 6;
const BAYER_DEMOSAIC: u32 = 2;
const XTRANS_DEMOSAIC: u32 = 3;
const CHROMA_RESAMPLE: u32 = 4;
const LANCZOS_LOBES: f32 = 3.0;
const BICUBIC_BEFORE: f32 = 1.0;
const BICUBIC_AFTER: f32 = 3.0;
const WARP_SLACK: f32 = 2.0;
const WARP_STEP: usize = 4;
const DISPLAY_SLACK: u32 = 2;

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}

pub fn lcm(a: u32, b: u32) -> u32 {
    a / gcd(a, b) * b
}

fn cfa_period(cfa: &str, cpp: usize) -> Option<u32> {
    if cpp != 1 {
        return None;
    }
    Some(if parse_xtrans(cfa).is_some() {
        XTRANS_PERIOD
    } else {
        BAYER_PERIOD
    })
}

pub fn alignment(cfa: &str, cpp: usize, block: usize) -> u32 {
    let cfa = cfa_period(cfa, cpp).map_or(1, |period| period / gcd(period, block as u32));
    lcm(cfa, HIST_ALIGN)
}

pub fn demosaic(cfa: &str, cpp: usize, block: usize) -> u32 {
    match cfa_period(cfa, cpp) {
        Some(_) if block > 1 => 0,
        Some(XTRANS_PERIOD) => XTRANS_DEMOSAIC,
        Some(_) => BAYER_DEMOSAIC,
        None => 0,
    }
}

fn wavelet(levels: usize) -> u32 {
    3 * ((1 << levels) - 1)
}

pub fn noise_reduction(luma: bool, chroma: bool) -> u32 {
    let luma = if luma { wavelet(LUMA_LEVELS) } else { 0 };
    let chroma = if chroma {
        2 * wavelet(CHROMA_LEVELS) + CHROMA_RESAMPLE
    } else {
        0
    };
    luma + chroma
}

pub fn capture_sharpen(radius: u32) -> u32 {
    (2 * ITERATIONS as u32 + 1) * radius + 1
}

pub fn display_halo(long_edge: u32) -> u32 {
    let selector = SelectorTaps::for_long_edge(long_edge as f32).reach();
    SHARPEN_KERNEL_HALF as u32 + 2 * selector + DISPLAY_SLACK
}

pub fn lanczos((start, end): (u32, u32), scale: f32, src_len: u32) -> (u32, u32) {
    let support = LANCZOS_LOBES * scale.max(1.0);
    let lo = ((start as f32 + 0.5) * scale - support - 0.5).floor() - 1.0;
    let hi = ((end as f32 - 0.5) * scale + support - 0.5).ceil() + 2.0;
    (lo.max(0.0) as u32, (hi.max(0.0) as u32).min(src_len))
}

fn edge_samples(start: u32, end: u32) -> impl Iterator<Item = f32> {
    (start..end)
        .step_by(WARP_STEP)
        .chain(once(end - 1))
        .map(|v| v as f32)
}

pub fn warp(rect: Rect, frame: (u32, u32), map: impl Fn(f32, f32) -> [f32; 2]) -> Rect {
    let (top, bottom) = (rect.y as f32, (rect.bottom() - 1) as f32);
    let (left, right) = (rect.x as f32, (rect.right() - 1) as f32);
    let rows = edge_samples(rect.x, rect.right()).flat_map(|x| [(x, top), (x, bottom)]);
    let cols = edge_samples(rect.y, rect.bottom()).flat_map(|y| [(left, y), (right, y)]);
    let (lo, hi) = rows.chain(cols).map(|(x, y)| map(x, y)).fold(
        ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]),
        |(lo, hi), s| {
            (
                [lo[0].min(s[0]), lo[1].min(s[1])],
                [hi[0].max(s[0]), hi[1].max(s[1])],
            )
        },
    );
    if !(lo[0].is_finite() && lo[1].is_finite() && hi[0].is_finite() && hi[1].is_finite()) {
        return Rect::full(frame);
    }
    let span = |lo: f32, hi: f32, n: u32| {
        let start = (lo - BICUBIC_BEFORE - WARP_SLACK)
            .floor()
            .clamp(0.0, (n - 1) as f32) as u32;
        let end = (hi + BICUBIC_AFTER + WARP_SLACK)
            .floor()
            .clamp(0.0, n as f32) as u32;
        (start, end.max(start + 1))
    };
    let (x0, x1) = span(lo[0], hi[0], frame.0);
    let (y0, y1) = span(lo[1], hi[1], frame.1);
    Rect::bounds(x0, y0, x1, y1)
}
