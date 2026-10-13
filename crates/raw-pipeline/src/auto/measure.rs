use super::weights::Weights;
use crate::math::{linear_srgb_to_oklab, srgb_to_linear};
use crate::ops::box_filter::{box_mean, min_filter};

const BINS: usize = 1024;
const BLACK_QUANTILE: f64 = 0.005;
const WHITE_QUANTILE: f64 = 0.995;
const SHADOW_RANGE: (f64, f64) = (0.03, 0.25);
const HIGHLIGHT_RANGE: (f64, f64) = (0.8, 0.97);
const CHROMA_FLOOR: f32 = 0.2;
const CHROMA_CEILING: f32 = 0.92;
const CHROMA_RADIUS: f64 = 0.01;
const LOCAL_RADIUS: f64 = 0.03;
const HAZE_RADIUS: f64 = 0.015;
const HAZE_WHITE_QUANTILE: f64 = 0.99;

pub(super) struct Image {
    pub(super) width: usize,
    pub(super) height: usize,
    lightness: Vec<f32>,
    green_red: Vec<f32>,
    blue_yellow: Vec<f32>,
    peak: Vec<f32>,
    dark: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Metrics {
    pub(super) key: f64,
    pub(super) black: f64,
    pub(super) white: f64,
    pub(super) highlight: f64,
    pub(super) shadow: f64,
    pub(super) spread: f64,
    pub(super) chroma: f64,
}

impl Image {
    pub(super) fn from_rgb8(bytes: &[u8], width: usize, height: usize) -> Self {
        let decode: Vec<f32> = (0..=255u8)
            .map(|v| srgb_to_linear(f32::from(v) / 255.0))
            .collect();
        let n = width * height;
        let mut lightness = Vec::with_capacity(n);
        let mut green_red = Vec::with_capacity(n);
        let mut blue_yellow = Vec::with_capacity(n);
        let mut peak = Vec::with_capacity(n);
        let mut dark = Vec::with_capacity(n);
        for px in bytes.chunks_exact(3).take(n) {
            let rgb = [
                decode[usize::from(px[0])],
                decode[usize::from(px[1])],
                decode[usize::from(px[2])],
            ];
            let [l, a, b] = linear_srgb_to_oklab(rgb);
            lightness.push(l);
            green_red.push(a);
            blue_yellow.push(b);
            peak.push(f32::from(px[0].max(px[1]).max(px[2])) / 255.0);
            dark.push(rgb[0].min(rgb[1]).min(rgb[2]));
        }
        Self {
            width,
            height,
            lightness,
            green_red,
            blue_yellow,
            peak,
            dark,
        }
    }

    pub(super) fn metrics(&self, weights: &Weights) -> Metrics {
        let lightness = Histogram::new(&self.lightness, &weights.scene);
        let bright = Histogram::new(&self.lightness, &weights.recoverable);
        let r = self.radius(CHROMA_RADIUS);
        let a = box_mean(&self.green_red, self.width, self.height, r);
        let b = box_mean(&self.blue_yellow, self.width, self.height, r);
        let chroma: Vec<f32> = a.iter().zip(b.iter()).map(|(a, b)| a.hypot(*b)).collect();
        let midtone: Vec<f32> = self
            .lightness
            .iter()
            .zip(&weights.scene)
            .map(|(&l, &w)| {
                if (CHROMA_FLOOR..=CHROMA_CEILING).contains(&l) {
                    w
                } else {
                    0.0
                }
            })
            .collect();
        Metrics {
            key: weighted_mean(&self.lightness, &weights.scene),
            black: lightness.quantile(BLACK_QUANTILE),
            white: bright.quantile(WHITE_QUANTILE),
            highlight: bright.mean_between(HIGHLIGHT_RANGE.0, HIGHLIGHT_RANGE.1),
            shadow: lightness.mean_between(SHADOW_RANGE.0, SHADOW_RANGE.1),
            spread: lightness.quantile(0.75) - lightness.quantile(0.25),
            chroma: weighted_mean(&chroma, &midtone),
        }
    }

    pub(super) fn clipped(&self, level: f32) -> Vec<bool> {
        self.peak.iter().map(|&p| p >= level).collect()
    }

    pub(super) fn clipped_share(&self, level: f32) -> f64 {
        let clipped = self.peak.iter().filter(|&&p| p >= level).count();
        clipped as f64 / self.peak.len().max(1) as f64
    }

    pub(super) fn local_contrast(&self, weights: &Weights) -> f64 {
        let r = self.radius(LOCAL_RADIUS);
        let once = box_mean(&self.lightness, self.width, self.height, r);
        let base = box_mean(&once, self.width, self.height, r);
        let detail: Vec<f32> = self
            .lightness
            .iter()
            .zip(base.iter())
            .map(|(l, b)| (l - b).abs())
            .collect();
        weighted_mean(&detail, &weights.scene)
    }

    pub(super) fn haze(&self, weights: &Weights) -> f64 {
        let r = self.radius(HAZE_RADIUS);
        let dark = min_filter(&self.dark, self.width, self.height, r);
        let veil = Histogram::new(&dark, &weights.scene).quantile(0.5);
        let white = Histogram::new(&self.peak, &weights.scene).quantile(HAZE_WHITE_QUANTILE);
        let white = f64::from(srgb_to_linear(white as f32)).max(1e-3);
        f64::from(srgb_to_linear(veil as f32)) / white
    }

    fn radius(&self, share: f64) -> usize {
        ((self.width.min(self.height) as f64 * share).round() as usize).max(1)
    }
}

fn weighted_mean(values: &[f32], weights: &[f32]) -> f64 {
    let (sum, mass) = values
        .iter()
        .zip(weights)
        .fold((0.0f64, 0.0f64), |(s, m), (&v, &w)| {
            (s + f64::from(v) * f64::from(w), m + f64::from(w))
        });
    if mass > 0.0 { sum / mass } else { 0.0 }
}

struct Histogram {
    bins: Vec<f64>,
    total: f64,
}

impl Histogram {
    fn new(values: &[f32], weights: &[f32]) -> Self {
        let mut bins = vec![0.0f64; BINS];
        for (&v, &w) in values.iter().zip(weights) {
            let i = ((v.clamp(0.0, 1.0) * BINS as f32) as usize).min(BINS - 1);
            bins[i] += f64::from(w);
        }
        let total = bins.iter().sum();
        Self { bins, total }
    }

    fn quantile(&self, q: f64) -> f64 {
        let target = q.clamp(0.0, 1.0) * self.total;
        let mut cumulative = 0.0;
        for (i, &m) in self.bins.iter().enumerate() {
            if m > 0.0 && cumulative + m >= target {
                return (i as f64 + (target - cumulative) / m) / BINS as f64;
            }
            cumulative += m;
        }
        1.0
    }

    fn mean_between(&self, q0: f64, q1: f64) -> f64 {
        let lo = q0 * self.total;
        let hi = q1 * self.total;
        let mut cumulative = 0.0;
        let mut sum = 0.0;
        let mut mass = 0.0;
        for (i, &m) in self.bins.iter().enumerate() {
            let overlap = ((cumulative + m).min(hi) - cumulative.max(lo)).max(0.0);
            sum += overlap * (i as f64 + 0.5) / BINS as f64;
            mass += overlap;
            cumulative += m;
        }
        if mass > 0.0 { sum / mass } else { 0.0 }
    }
}
