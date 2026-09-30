use std::sync::Arc;

use wgpu::{Buffer, Texture};

use super::profile::{luma_block, packed};
use super::sweep::{NrSweep, groups, hist_groups};
use crate::gpu::passes::nr::{
    NR_INIT_READ, NR_INIT_ZERO, NR_LUMA_FORMAT, NrHistParams, NrPasses, NrShrinkParams,
    NrSizeParams,
};
use crate::ops::denoise::LUMA_LEVELS;
use crate::ops::denoise::estimate::{HIST_LEN, NoiseCurve};
use crate::ops::denoise::shrink::LevelParams;

fn planes(
    sweep: &mut NrSweep<'_>,
    p: &NrPasses,
    src: &Texture,
    (w, h): (u32, u32),
) -> [Arc<Texture>; 2] {
    let cur = sweep.scratch(NR_LUMA_FORMAT, (w, h), "nr-luma-cur");
    let next = sweep.scratch(NR_LUMA_FORMAT, (w, h), "nr-luma-next");
    let params = NrSizeParams {
        size: [w, h],
        _pad: [0; 2],
    };
    sweep.run(
        "nr-luma-init",
        &p.luma_init,
        &params,
        &[src, &cur],
        None,
        groups((w, h)),
    );
    [cur, next]
}

pub(super) fn estimate(
    sweep: &mut NrSweep<'_>,
    p: &NrPasses,
    src: &Texture,
    dims: (u32, u32),
    hist: &Buffer,
) {
    let [mut cur, mut next] = planes(sweep, p, src, dims);
    let tmp = sweep.scratch(NR_LUMA_FORMAT, dims, "nr-luma-tmp");
    for s in 0..LUMA_LEVELS {
        sweep.atrous(&p.luma, [&cur, &tmp, &next], dims, 1 << s);
        let params = NrHistParams {
            size: [dims.0, dims.1],
            lo_size: [dims.0, dims.1],
            lo: 0,
            count: 1,
            offset: (luma_block(s) * HIST_LEN) as u32,
            fine: 0,
        };
        sweep.run(
            "nr-luma-hist",
            &p.hist,
            &params,
            &[&cur, &next, &next],
            Some(hist),
            hist_groups(dims),
        );
        std::mem::swap(&mut cur, &mut next);
    }
}

pub(super) fn render(
    sweep: &mut NrSweep<'_>,
    p: &NrPasses,
    [src, dst]: [&Texture; 2],
    dims: (u32, u32),
    curves: &[NoiseCurve; LUMA_LEVELS],
    levels: [LevelParams; LUMA_LEVELS],
) {
    let [mut cur, mut next] = planes(sweep, p, src, dims);
    let mut acc = sweep.scratch(NR_LUMA_FORMAT, dims, "nr-luma-acc");
    let mut spare = sweep.scratch(NR_LUMA_FORMAT, dims, "nr-luma-spare");
    for (s, level) in levels.into_iter().enumerate() {
        let step = 1 << s;
        sweep.atrous(&p.luma, [&cur, &spare, &next], dims, step);
        let params = NrShrinkParams {
            curves: packed(&curves[s..=s]),
            size: [dims.0, dims.1],
            step,
            lo: 0,
            count: 1,
            init: if s == 0 { NR_INIT_ZERO } else { NR_INIT_READ },
            lambda: level.lambda,
            mu: level.mu,
            keep: level.keep,
            _pad: [0; 3],
        };
        let acc_in = if s == 0 { &cur } else { &acc };
        sweep.run(
            "nr-luma-shrink",
            &p.luma.shrink,
            &params,
            &[&cur, &next, acc_in, &spare],
            None,
            groups(dims),
        );
        std::mem::swap(&mut acc, &mut spare);
        std::mem::swap(&mut cur, &mut next);
    }
    let params = NrSizeParams {
        size: [dims.0, dims.1],
        _pad: [0; 2],
    };
    sweep.run(
        "nr-luma-finish",
        &p.luma_finish,
        &params,
        &[src, &acc, dst],
        None,
        groups(dims),
    );
}
