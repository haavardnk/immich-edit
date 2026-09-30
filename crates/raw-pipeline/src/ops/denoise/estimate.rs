use rayon::prelude::*;

pub(crate) const LUM_BINS: usize = 16;
pub(crate) const MAG_BINS: usize = 96;
pub(crate) const MAG_LOG2_MIN: f32 = -20.0;
pub(crate) const MAG_BINS_PER_OCTAVE: f32 = 4.0;
pub(crate) const HIST_LEN: usize = LUM_BINS * MAG_BINS;
pub(crate) const SAMPLE_STRIDE: usize = 2;
const MIN_SAMPLES: u32 = 256;
const MAD_TO_SIGMA: f32 = 1.4826;
const LEVEL_RATIO_CAP: f32 = 0.3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct NoiseCurve {
    pub a: f32,
    pub b: f32,
}

impl NoiseCurve {
    #[inline(always)]
    pub(crate) fn variance(self, y: f32) -> f32 {
        (self.a + self.b * y.max(0.0)).max(0.0)
    }
}

#[inline(always)]
pub(crate) fn lum_bin(y: f32) -> usize {
    ((y.clamp(0.0, 1.0).sqrt() * LUM_BINS as f32) as usize).min(LUM_BINS - 1)
}

#[inline(always)]
pub(crate) fn mag_bin(m: f32) -> Option<usize> {
    if m.is_nan() || m <= 0.0 {
        return None;
    }
    let b = ((m.log2() - MAG_LOG2_MIN) * MAG_BINS_PER_OCTAVE).floor();
    Some(b.clamp(0.0, (MAG_BINS - 1) as f32) as usize)
}

pub(crate) fn histogram<F>(w: usize, h: usize, sample: F) -> Vec<u32>
where
    F: Fn(usize, usize) -> (f32, f32) + Sync,
{
    (0..h)
        .into_par_iter()
        .step_by(SAMPLE_STRIDE)
        .fold(
            || vec![0u32; HIST_LEN],
            |mut hist, y| {
                for x in (0..w).step_by(SAMPLE_STRIDE) {
                    let (d, reference) = sample(x, y);
                    if let Some(m) = mag_bin(d.abs()) {
                        hist[lum_bin(reference) * MAG_BINS + m] += 1;
                    }
                }
                hist
            },
        )
        .reduce(
            || vec![0u32; HIST_LEN],
            |mut a, b| {
                a.iter_mut().zip(&b).for_each(|(x, y)| *x += y);
                a
            },
        )
}

pub(crate) fn fit(hist: &[u32], coarser_than: Option<NoiseCurve>) -> NoiseCurve {
    let points: Vec<[f32; 3]> = hist
        .chunks_exact(MAG_BINS)
        .enumerate()
        .filter_map(|(i, bins)| {
            let n: u32 = bins.iter().sum();
            if n < MIN_SAMPLES {
                return None;
            }
            let sigma = median(bins, n) * MAD_TO_SIGMA;
            let y = ((i as f32 + 0.5) / LUM_BINS as f32).powi(2);
            Some([y, sigma * sigma, (n as f32).sqrt()])
        })
        .collect();
    let curve = line_fit(&points);
    let Some(prev) = coarser_than else {
        return curve;
    };
    NoiseCurve {
        a: curve.a.min(prev.a * LEVEL_RATIO_CAP),
        b: curve.b.min(prev.b * LEVEL_RATIO_CAP),
    }
}

fn median(bins: &[u32], n: u32) -> f32 {
    let target = n as f32 * 0.5;
    let mut cum = 0.0f32;
    for (k, &c) in bins.iter().enumerate() {
        let c = c as f32;
        if c > 0.0 && cum + c >= target {
            let frac = (target - cum) / c;
            return (MAG_LOG2_MIN + (k as f32 + frac) / MAG_BINS_PER_OCTAVE).exp2();
        }
        cum += c;
    }
    0.0
}

fn line_fit(points: &[[f32; 3]]) -> NoiseCurve {
    let sw: f32 = points.iter().map(|p| p[2]).sum();
    if sw <= 0.0 {
        return NoiseCurve::default();
    }
    let my = points.iter().map(|p| p[2] * p[0]).sum::<f32>() / sw;
    let mv = points.iter().map(|p| p[2] * p[1]).sum::<f32>() / sw;
    let sxx: f32 = points
        .iter()
        .map(|p| p[2] * (p[0] - my) * (p[0] - my))
        .sum();
    let sxy: f32 = points
        .iter()
        .map(|p| p[2] * (p[0] - my) * (p[1] - mv))
        .sum();
    let b = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    if b <= 0.0 {
        return NoiseCurve { a: mv, b: 0.0 };
    }
    let a = mv - b * my;
    if a >= 0.0 {
        return NoiseCurve { a, b };
    }
    let syy: f32 = points.iter().map(|p| p[2] * p[0] * p[0]).sum();
    let syv: f32 = points.iter().map(|p| p[2] * p[0] * p[1]).sum();
    NoiseCurve {
        a: 0.0,
        b: syv / syy,
    }
}
