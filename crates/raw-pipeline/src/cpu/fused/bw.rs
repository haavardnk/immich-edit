use crate::math::{luma, smoothstep};
use crate::ops::bw::{BW_MIX_EV, BW_TINT_BALANCE_RANGE, BW_TINT_PIVOT, BW_TINT_WIDTH};

#[derive(Clone, Copy)]
pub struct BwParams<'a> {
    pub mix: &'a [f32; 6],
    pub shadow_tint: &'a [f32; 3],
    pub highlight_tint: &'a [f32; 3],
    pub balance: f32,
}

#[inline(always)]
fn hue(r: f32, g: f32, b: f32, max: f32, d: f32) -> f32 {
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    h * 60.0
}

#[inline(always)]
fn channel_mix(mix: &[f32; 6], hue: f32) -> f32 {
    let h = hue.rem_euclid(360.0) / 60.0;
    let i = (h as usize).min(5);
    let t = h - i as f32;
    mix[i] * (1.0 - t) + mix[(i + 1) % 6] * t
}

#[inline(always)]
pub fn apply_bw(p: BwParams<'_>, r: &mut f32, g: &mut f32, b: &mut f32) {
    let cr = r.max(0.0);
    let cg = g.max(0.0);
    let cb = b.max(0.0);
    let y = luma(cr, cg, cb);
    let max = cr.max(cg).max(cb);
    let d = max - cr.min(cg).min(cb);
    let weight = if d > 1e-6 && max > 1e-6 {
        d / max * channel_mix(p.mix, hue(cr, cg, cb, max, d))
    } else {
        0.0
    };
    let gray = y * (BW_MIX_EV * weight).exp2();
    let pivot = BW_TINT_PIVOT - BW_TINT_BALANCE_RANGE * p.balance;
    let wh = smoothstep(
        pivot - BW_TINT_WIDTH,
        pivot + BW_TINT_WIDTH,
        gray.clamp(0.0, 1.0).sqrt(),
    );
    let tinted = |i: usize| {
        (gray * (1.0 + p.shadow_tint[i] * (1.0 - wh) + p.highlight_tint[i] * wh)).max(0.0)
    };
    *r = tinted(0);
    *g = tinted(1);
    *b = tinted(2);
}
