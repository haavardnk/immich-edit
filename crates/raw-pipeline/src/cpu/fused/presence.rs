use std::sync::Arc;

use crate::math::fast;
use crate::math::{luma, smoothstep};

#[inline(always)]
pub fn apply_white_balance(coeffs: &[f32; 3], r: &mut f32, g: &mut f32, b: &mut f32) {
    *r *= coeffs[0];
    *g *= coeffs[1];
    *b *= coeffs[2];
}

#[derive(Clone, Copy)]
pub struct PresenceParams<'a> {
    pub texture: f32,
    pub clarity: f32,
    pub exposure: f32,
    pub texture_blur: Option<&'a Arc<Vec<f32>>>,
    pub clarity_guide: Option<&'a Arc<Vec<f32>>>,
}

#[inline(always)]
pub fn apply_presence(p: PresenceParams<'_>, i: usize, r: &mut f32, g: &mut f32, b: &mut f32) {
    let y0 = luma(*r, *g, *b);
    let y0c = y0.max(1e-5);
    let mut log_gain = 0.0f32;
    if let Some(buf) = p.texture_blur {
        log_gain += p.texture * fast::log2(y0c / buf[i].max(1e-5));
    }
    if let Some(buf) = p.clarity_guide {
        let mt = crate::ops::presence::clarity_midtones(y0, p.exposure);
        let ratio = (1.0 - buf[2 * i]) * fast::log2(y0c) - buf[2 * i + 1];
        let gate = smoothstep(0.015, 0.12, ratio.abs());
        log_gain += p.clarity * mt * gate * ratio;
    }
    let new_y = y0 * fast::exp2(log_gain);
    let goal = new_y.max(0.0);
    if y0 <= 1e-5 {
        *r = goal;
        *g = goal;
        *b = goal;
        return;
    }
    let scale = goal / y0;
    *r = (*r * scale).max(0.0);
    *g = (*g * scale).max(0.0);
    *b = (*b * scale).max(0.0);
}
