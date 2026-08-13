use crate::histogram::{self, Histogram};
use crate::math::luma;

pub(super) type HistBins = (
    [u32; histogram::BINS],
    [u32; histogram::BINS],
    [u32; histogram::BINS],
    [u32; histogram::BINS],
);

pub(super) fn zero_bins() -> (HistBins, HistBins) {
    let empty = || {
        (
            [0; histogram::BINS],
            [0; histogram::BINS],
            [0; histogram::BINS],
            [0; histogram::BINS],
        )
    };
    (empty(), empty())
}

pub(super) fn fold_linear(acc: &mut HistBins, lr: f32, lg: f32, lb: f32) {
    let li = luma(lr, lg, lb).clamp(0.0, 1.0);
    acc.0[((lr.clamp(0.0, 1.0) * 255.0) as usize).min(histogram::BINS - 1)] += 1;
    acc.1[((lg.clamp(0.0, 1.0) * 255.0) as usize).min(histogram::BINS - 1)] += 1;
    acc.2[((lb.clamp(0.0, 1.0) * 255.0) as usize).min(histogram::BINS - 1)] += 1;
    acc.3[((li * 255.0) as usize).min(histogram::BINS - 1)] += 1;
}

pub(super) fn fold_display(acc: &mut HistBins, ur: u8, ug: u8, ub: u8) {
    let li = luma(ur as f32, ug as f32, ub as f32) as usize;
    acc.0[ur as usize] += 1;
    acc.1[ug as usize] += 1;
    acc.2[ub as usize] += 1;
    acc.3[li.min(histogram::BINS - 1)] += 1;
}

pub(super) fn merge_bins(mut a: HistBins, b: HistBins) -> HistBins {
    for i in 0..histogram::BINS {
        a.0[i] += b.0[i];
        a.1[i] += b.1[i];
        a.2[i] += b.2[i];
        a.3[i] += b.3[i];
    }
    a
}

pub(super) fn bins_to_histogram(bins: HistBins) -> Histogram {
    Histogram {
        r: bins.0.to_vec(),
        g: bins.1.to_vec(),
        b: bins.2.to_vec(),
        l: bins.3.to_vec(),
    }
}
