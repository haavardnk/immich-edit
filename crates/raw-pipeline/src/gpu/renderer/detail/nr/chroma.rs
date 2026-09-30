use std::sync::Arc;

use wgpu::{Buffer, Texture};

use super::profile::{FINE_BLOCK, NoiseProfile, chroma_block, packed};
use super::sweep::{NrSweep, groups, half, hist_groups};
use crate::gpu::passes::nr::{
    NR_CHROMA_FORMAT, NR_INIT_CARRY_REFERENCE, NR_INIT_READ, NrApplyParams, NrHistParams, NrPasses,
    NrShrinkParams, NrSizeParams,
};
use crate::ops::denoise::CHROMA_LEVELS;
use crate::ops::denoise::estimate::HIST_LEN;
use crate::ops::denoise::shrink::LevelParams;

const PB: u32 = 1;
const CHANNELS: u32 = 2;

fn planes(
    sweep: &mut NrSweep<'_>,
    p: &NrPasses,
    src: &Texture,
    dims: (u32, u32),
) -> [Arc<Texture>; 3] {
    let lo = half(dims);
    let cur = sweep.scratch(NR_CHROMA_FORMAT, lo, "nr-chroma-cur");
    let next = sweep.scratch(NR_CHROMA_FORMAT, lo, "nr-chroma-next");
    let base = sweep.scratch(NR_CHROMA_FORMAT, lo, "nr-chroma-base");
    let params = NrSizeParams {
        size: [dims.0, dims.1],
        _pad: [0; 2],
    };
    sweep.run(
        "nr-chroma-down",
        &p.chroma_down,
        &params,
        &[src, &cur, &base],
        None,
        groups(lo),
    );
    [cur, next, base]
}

fn hist_params(size: (u32, u32), lo: (u32, u32), block: usize, fine: u32) -> NrHistParams {
    NrHistParams {
        size: [size.0, size.1],
        lo_size: [lo.0, lo.1],
        lo: PB,
        count: CHANNELS,
        offset: (block * HIST_LEN) as u32,
        fine,
    }
}

pub(super) fn estimate(
    sweep: &mut NrSweep<'_>,
    p: &NrPasses,
    src: &Texture,
    dims: (u32, u32),
    hist: &Buffer,
) {
    let lo = half(dims);
    let [mut cur, mut next, base] = planes(sweep, p, src, dims);
    let tmp = sweep.scratch(NR_CHROMA_FORMAT, lo, "nr-chroma-tmp");
    for s in 0..CHROMA_LEVELS {
        sweep.atrous(&p.chroma, [&cur, &tmp, &next], lo, 1 << s);
        if s == 0 {
            let params = hist_params(dims, lo, FINE_BLOCK, 1);
            sweep.run(
                "nr-fine-hist",
                &p.hist,
                &params,
                &[src, &base, &next],
                Some(hist),
                hist_groups(dims),
            );
        }
        let params = hist_params(lo, lo, chroma_block(s), 0);
        sweep.run(
            "nr-chroma-hist",
            &p.hist,
            &params,
            &[&cur, &next, &next],
            Some(hist),
            hist_groups(lo),
        );
        std::mem::swap(&mut cur, &mut next);
    }
}

pub(super) fn render(
    sweep: &mut NrSweep<'_>,
    p: &NrPasses,
    [src, dst]: [&Texture; 2],
    dims: (u32, u32),
    profile: &NoiseProfile,
    levels: [LevelParams; CHROMA_LEVELS + 1],
) {
    let lo = half(dims);
    let [mut cur, mut next, base] = planes(sweep, p, src, dims);
    let mut den = sweep.scratch(NR_CHROMA_FORMAT, lo, "nr-chroma-den");
    let mut spare = sweep.scratch(NR_CHROMA_FORMAT, lo, "nr-chroma-spare");
    for (s, level) in levels[1..].iter().enumerate() {
        let step = 1 << s;
        sweep.atrous(&p.chroma, [&cur, &spare, &next], lo, step);
        let params = NrShrinkParams {
            curves: packed(&profile.chroma[s]),
            size: [lo.0, lo.1],
            step,
            lo: PB,
            count: CHANNELS,
            init: if s == 0 {
                NR_INIT_CARRY_REFERENCE
            } else {
                NR_INIT_READ
            },
            lambda: level.lambda,
            mu: level.mu,
            keep: level.keep,
            _pad: [0; 3],
        };
        let den_in = if s == 0 { &base } else { &den };
        sweep.run(
            "nr-chroma-shrink",
            &p.chroma.shrink,
            &params,
            &[&cur, &next, den_in, &spare],
            None,
            groups(lo),
        );
        std::mem::swap(&mut den, &mut spare);
        std::mem::swap(&mut cur, &mut next);
    }
    let params = NrApplyParams {
        curves: packed(&profile.fine),
        size: [dims.0, dims.1],
        lo_size: [lo.0, lo.1],
        lambda: levels[0].lambda,
        mu: levels[0].mu,
        keep: levels[0].keep,
        _pad: 0,
    };
    sweep.run(
        "nr-chroma-apply",
        &p.chroma_apply,
        &params,
        &[src, &base, &den, dst],
        None,
        groups(dims),
    );
}
