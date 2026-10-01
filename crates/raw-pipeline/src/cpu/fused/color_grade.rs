use crate::math::{linear_srgb_to_oklab, oklab_to_linear_srgb, smoothstep};
use crate::ops::color_grade::{
    COLOR_GRADE_FEATHER_BASE, COLOR_GRADE_FEATHER_RANGE, COLOR_GRADE_PIVOT_BASE,
    COLOR_GRADE_PIVOT_RANGE,
};

#[inline(always)]
fn cg_weights(y: f32, balance: f32, blend: f32) -> (f32, f32, f32) {
    let pivot = COLOR_GRADE_PIVOT_BASE + COLOR_GRADE_PIVOT_RANGE * balance;
    let feather = COLOR_GRADE_FEATHER_BASE + COLOR_GRADE_FEATHER_RANGE * blend;
    let s_hi = (pivot + feather * 0.5).clamp(0.001, 0.999);
    let s_lo = (pivot - feather - feather * 0.5).clamp(0.0, s_hi - 0.001);
    let h_lo = (pivot - feather * 0.5).clamp(0.001, 0.999);
    let h_hi = (pivot + feather + feather * 0.5).clamp(h_lo + 0.001, 1.0);
    let shadow = 1.0 - smoothstep(s_lo, s_hi, y);
    let highlight = smoothstep(h_lo, h_hi, y);
    let mid = (1.0 - shadow - highlight).max(0.0);
    (shadow, mid, highlight)
}

#[inline(always)]
pub fn apply_color_grade(
    regions: &[[f32; 3]; 4],
    balance: f32,
    blend: f32,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) {
    let [l, a, bb] = linear_srgb_to_oklab([*r, *g, *b]);
    let (ws, wm, wh) = cg_weights(l.clamp(0.0, 1.0), balance, blend);
    let [s, m, h, global] = regions;
    let mix = |i: usize| ws * s[i] + wm * m[i] + wh * h[i] + global[i];
    let lit = l.max(0.0);
    let gain = mix(2).exp2();
    [*r, *g, *b] = oklab_to_linear_srgb([l, a + lit * mix(0), bb + lit * mix(1)]).map(|v| v * gain);
}
