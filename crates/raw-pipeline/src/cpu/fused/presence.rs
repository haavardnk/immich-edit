use std::sync::Arc;

use crate::math::smoothstep;
use crate::tone::shared::{HL_RECONSTRUCT_BIAS, HL_RECONSTRUCT_KNEE, RAW_SENSOR_WHITE};

#[inline(always)]
pub fn apply_white_balance(
    coeffs: &[f32; 3],
    reconstruct: bool,
    r: &mut f32,
    g: &mut f32,
    b: &mut f32,
) {
    let pr = *r;
    let pg = *g;
    let pb = *b;
    let mut wr = pr * coeffs[0];
    let mut wg = pg * coeffs[1];
    let mut wb = pb * coeffs[2];
    if reconstruct {
        let cr = smoothstep(HL_RECONSTRUCT_KNEE, RAW_SENSOR_WHITE, pr);
        let cg = smoothstep(HL_RECONSTRUCT_KNEE, RAW_SENSOR_WHITE, pg);
        let cb = smoothstep(HL_RECONSTRUCT_KNEE, RAW_SENSOR_WHITE, pb);
        if cr.max(cg).max(cb) > 0.0 {
            let ur = 1.0 - cr;
            let ug = 1.0 - cg;
            let ub = 1.0 - cb;
            let wmax = wr.max(wg).max(wb);
            let target = (ur * wr + ug * wg + ub * wb + HL_RECONSTRUCT_BIAS * wmax)
                / (ur + ug + ub + HL_RECONSTRUCT_BIAS);
            if wr < target {
                wr += (target - wr) * cr;
            }
            if wg < target {
                wg += (target - wg) * cg;
            }
            if wb < target {
                wb += (target - wb) * cb;
            }
        }
    }
    *r = wr;
    *g = wg;
    *b = wb;
}

#[derive(Clone, Copy)]
pub struct PresenceParams<'a> {
    pub texture: f32,
    pub clarity: f32,
    pub texture_blur: Option<&'a Arc<Vec<f32>>>,
    pub clarity_blur: Option<&'a Arc<Vec<f32>>>,
}

#[inline(always)]
pub fn apply_presence(p: PresenceParams<'_>, i: usize, r: &mut f32, g: &mut f32, b: &mut f32) {
    let y0 = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
    let y0c = y0.max(1e-5);
    let mut log_gain = 0.0f32;
    if let Some(buf) = p.texture_blur {
        log_gain += p.texture * (y0c / buf[i].max(1e-5)).log2();
    }
    if let Some(buf) = p.clarity_blur {
        let mt = smoothstep(0.0, 0.1, y0)
            * (1.0 - smoothstep(0.9, 1.0, y0))
            * (1.0 - (2.0 * y0 - 1.0).abs()).max(0.0);
        let ratio = (y0c / buf[i].max(1e-5)).log2();
        let gate = smoothstep(0.015, 0.12, ratio.abs());
        log_gain += p.clarity * mt * gate * ratio;
    }
    let new_y = y0 * log_gain.exp2();
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
