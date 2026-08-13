use crate::tone::shared::{LUMA_B, LUMA_G, LUMA_R};

#[inline(always)]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline(always)]
pub fn luma(r: f32, g: f32, b: f32) -> f32 {
    LUMA_R * r + LUMA_G * g + LUMA_B * b
}

#[inline(always)]
pub fn hue_dist(a: f32, b: f32) -> f32 {
    let d = (a - b).rem_euclid(360.0);
    d.min(360.0 - d)
}

#[inline(always)]
pub fn srgb_to_linear(v: f32) -> f32 {
    let c = v.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
