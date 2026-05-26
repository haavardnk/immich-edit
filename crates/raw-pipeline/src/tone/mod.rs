use crate::edits::{OutputEdits, TonemapKind};

pub mod agx;
pub mod shared;
pub mod wgsl;

use shared::{
    HIGHLIGHT_KNEE, OETF_LUT_SIZE, S_CURVE_BLEND, SRGB_OETF_GAMMA, SRGB_OETF_GAMMA_OFFSET,
    SRGB_OETF_GAMMA_SCALE, SRGB_OETF_LINEAR_CUTOFF, SRGB_OETF_LINEAR_SLOPE,
};

fn oetf_lut() -> &'static [f32; OETF_LUT_SIZE + 1] {
    static LUT: std::sync::OnceLock<[f32; OETF_LUT_SIZE + 1]> = std::sync::OnceLock::new();
    LUT.get_or_init(|| {
        let mut t = [0.0f32; OETF_LUT_SIZE + 1];
        for (i, slot) in t.iter_mut().enumerate() {
            let v = i as f32 / OETF_LUT_SIZE as f32;
            *slot = srgb_oetf_scalar(v);
        }
        t
    })
}

pub fn srgb_oetf(v: f32) -> f32 {
    let lut = oetf_lut();
    let scaled = v.clamp(0.0, 1.0) * OETF_LUT_SIZE as f32;
    let idx = scaled as usize;
    let frac = scaled - idx as f32;
    let lo = lut[idx];
    let hi = lut[(idx + 1).min(OETF_LUT_SIZE)];
    lo + (hi - lo) * frac
}

pub fn srgb_oetf_scalar(v: f32) -> f32 {
    if v <= SRGB_OETF_LINEAR_CUTOFF {
        SRGB_OETF_LINEAR_SLOPE * v
    } else {
        SRGB_OETF_GAMMA_SCALE * v.powf(SRGB_OETF_GAMMA) - SRGB_OETF_GAMMA_OFFSET
    }
}

fn soft_clip_high(v: f32) -> f32 {
    if v <= HIGHLIGHT_KNEE {
        return v;
    }
    let headroom = 1.0 - HIGHLIGHT_KNEE;
    let excess = v - HIGHLIGHT_KNEE;
    HIGHLIGHT_KNEE + headroom * (excess / (excess + headroom))
}

pub fn default_scalar(v: f32) -> f32 {
    let lin = if v <= 0.0 { 0.0 } else { soft_clip_high(v) };
    let srgb = srgb_oetf(lin);
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    srgb + (s - srgb) * S_CURVE_BLEND
}

pub fn apply_default_rgb(rgb: [f32; 3]) -> [f32; 3] {
    [
        default_scalar(rgb[0]),
        default_scalar(rgb[1]),
        default_scalar(rgb[2]),
    ]
}

pub fn apply_rgb(rgb: [f32; 3], output: OutputEdits) -> [f32; 3] {
    match output.tonemap {
        TonemapKind::Default => apply_default_rgb(rgb),
        TonemapKind::Agx => agx::apply_rgb(rgb),
    }
}

pub fn apply_display_luma(rgb: [f32; 3], output: OutputEdits) -> f32 {
    let display = apply_rgb(rgb, output);
    0.2126 * display[0] + 0.7152 * display[1] + 0.0722 * display[2]
}

pub const fn tonemap_kind_index(kind: TonemapKind) -> u32 {
    match kind {
        TonemapKind::Default => shared::TONE_KIND_DEFAULT,
        TonemapKind::Agx => shared::TONE_KIND_AGX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_endpoints() {
        if default_scalar(0.0).abs() > 1e-6 {
            panic!("default(0) should be 0");
        }
        let one = default_scalar(1.0);
        if !(0.97..=1.0).contains(&one) {
            panic!("default(1) should be ~1, got {one}");
        }
    }

    #[test]
    fn default_monotonic_sample() {
        let mut prev = -1.0f32;
        for i in 0..=64 {
            let x = i as f32 / 64.0;
            let y = default_scalar(x);
            if y < prev - 1e-6 {
                panic!("non-monotone at {x}: {y} < {prev}");
            }
            prev = y;
        }
    }
}
