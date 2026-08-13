use crate::math::smoothstep;
use crate::ops::color_grade::{
    COLOR_GRADE_FEATHER_BASE, COLOR_GRADE_FEATHER_RANGE, COLOR_GRADE_PIVOT_BASE,
    COLOR_GRADE_PIVOT_RANGE, COLOR_GRADE_STRENGTH,
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

#[derive(Clone, Copy)]
pub struct ColorGradeParams<'a> {
    pub s_off: &'a [f32; 3],
    pub s_lum: f32,
    pub m_off: &'a [f32; 3],
    pub m_lum: f32,
    pub h_off: &'a [f32; 3],
    pub h_lum: f32,
    pub g_off: &'a [f32; 3],
    pub g_lum: f32,
    pub balance: f32,
    pub blend: f32,
}

#[inline(always)]
pub fn apply_color_grade(p: ColorGradeParams<'_>, r: &mut f32, g: &mut f32, b: &mut f32) {
    let y = (0.2126 * *r + 0.7152 * *g + 0.0722 * *b).clamp(0.0, 1.0);
    let (ws, wm, wh) = cg_weights(y, p.balance, p.blend);
    let or =
        (ws * p.s_off[0] + wm * p.m_off[0] + wh * p.h_off[0] + p.g_off[0]) * COLOR_GRADE_STRENGTH;
    let og =
        (ws * p.s_off[1] + wm * p.m_off[1] + wh * p.h_off[1] + p.g_off[1]) * COLOR_GRADE_STRENGTH;
    let ob =
        (ws * p.s_off[2] + wm * p.m_off[2] + wh * p.h_off[2] + p.g_off[2]) * COLOR_GRADE_STRENGTH;
    let lum = (ws * p.s_lum + wm * p.m_lum + wh * p.h_lum + p.g_lum) * COLOR_GRADE_STRENGTH;
    *r = (*r + or + lum).max(0.0);
    *g = (*g + og + lum).max(0.0);
    *b = (*b + ob + lum).max(0.0);
}
