use super::shared::{
    AGX_DESAT_CEIL, AGX_DESAT_KNEE, AGX_IN_B, AGX_IN_G, AGX_IN_R, AGX_MAX_EV, AGX_MIN_EV,
    AGX_MIN_LIN, AGX_OUT_B, AGX_OUT_G, AGX_OUT_R, AGX_SIGMOID_COEFFS,
};

const RANGE_EV: f32 = AGX_MAX_EV - AGX_MIN_EV;

fn mat_mul(c: [f32; 3], r: [f32; 3], g: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        c[0] * r[0] + c[1] * r[1] + c[2] * r[2],
        c[0] * g[0] + c[1] * g[1] + c[2] * g[2],
        c[0] * b[0] + c[1] * b[1] + c[2] * b[2],
    ]
}

fn log2_normalize(v: f32) -> f32 {
    let clamped = v.max(AGX_MIN_LIN);
    ((clamped.log2() - AGX_MIN_EV) / RANGE_EV).clamp(0.0, 1.0)
}

fn sigmoid(x: f32) -> f32 {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    AGX_SIGMOID_COEFFS[0] * x6 * x
        + AGX_SIGMOID_COEFFS[1] * x6
        + AGX_SIGMOID_COEFFS[2] * x4 * x
        + AGX_SIGMOID_COEFFS[3] * x4
        + AGX_SIGMOID_COEFFS[4] * x2 * x
        + AGX_SIGMOID_COEFFS[5] * x2
        + AGX_SIGMOID_COEFFS[6] * x
        + AGX_SIGMOID_COEFFS[7]
}

fn highlight_desat(rgb: [f32; 3]) -> [f32; 3] {
    let m = rgb[0].max(rgb[1]).max(rgb[2]);
    if m <= AGX_DESAT_KNEE {
        return rgb;
    }
    let y = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    let t_lin = ((m - AGX_DESAT_KNEE) / (AGX_DESAT_CEIL - AGX_DESAT_KNEE)).clamp(0.0, 1.0);
    let t = t_lin * t_lin * (3.0 - 2.0 * t_lin);
    let k = 1.0 - t;
    [
        y + (rgb[0] - y) * k,
        y + (rgb[1] - y) * k,
        y + (rgb[2] - y) * k,
    ]
}

pub fn apply_rgb(rgb: [f32; 3]) -> [f32; 3] {
    let compressed = highlight_desat(rgb);
    let working = mat_mul(compressed, AGX_IN_R, AGX_IN_G, AGX_IN_B);
    let log_rgb = [
        log2_normalize(working[0]),
        log2_normalize(working[1]),
        log2_normalize(working[2]),
    ];
    let sigmoided = [
        sigmoid(log_rgb[0]),
        sigmoid(log_rgb[1]),
        sigmoid(log_rgb[2]),
    ];
    let display_linear = mat_mul(sigmoided, AGX_OUT_R, AGX_OUT_G, AGX_OUT_B);
    [
        display_linear[0].clamp(0.0, 1.0),
        display_linear[1].clamp(0.0, 1.0),
        display_linear[2].clamp(0.0, 1.0),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_maps_to_zero_ish() {
        let out = apply_rgb([0.0, 0.0, 0.0]);
        for c in out {
            if c > 0.01 {
                panic!("agx(0) channel out of range: {c}");
            }
        }
    }

    #[test]
    fn grey_ramp_monotonic() {
        let mut prev = -1.0f32;
        for i in 0..64 {
            let x = (i as f32 / 63.0) * 4.0;
            let y = apply_rgb([x, x, x]);
            let l = 0.2126 * y[0] + 0.7152 * y[1] + 0.0722 * y[2];
            if l < prev - 1e-4 {
                panic!("non-monotone at {x}: {l} < {prev}");
            }
            prev = l;
        }
    }

    #[test]
    fn hdr_bounded() {
        for ev in -8..=8 {
            let x = (2.0f32).powi(ev);
            let out = apply_rgb([x, x, x]);
            for c in out {
                if !c.is_finite() || !(0.0..=1.0).contains(&c) {
                    panic!("agx out of range at ev={ev}: {c}");
                }
            }
        }
    }

    #[test]
    fn saturated_primary_compresses_toward_white() {
        let out = apply_rgb([8.0, 0.0, 0.0]);
        if out[1] < 0.05 || out[2] < 0.02 {
            panic!("expected highlight desat to bleed into G/B, got {out:?}");
        }
    }

    #[test]
    fn near_white_fringe_stays_neutral() {
        let out = apply_rgb([1.0, 0.95, 1.0]);
        let max = out[0].max(out[1]).max(out[2]);
        let min = out[0].min(out[1]).min(out[2]);
        if max - min > 0.05 {
            panic!("near-white fringe should desaturate, got {out:?}");
        }
    }
}
