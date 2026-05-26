const MIN_EV: f32 = -10.0;
const MAX_EV: f32 = 6.5;
const RANGE_EV: f32 = MAX_EV - MIN_EV;
const MIN_LIN: f32 = 0.000_976_562_5;

const DESAT_KNEE: f32 = 0.7;
const DESAT_CEIL: f32 = 2.0;

const IN_R: [f32; 3] = [0.842_479, 0.078_411_03, 0.087_828_82];
const IN_G: [f32; 3] = [0.042_328_99, 0.878_869_2, 0.078_843_8];
const IN_B: [f32; 3] = [0.042_424_29, 0.078_843_8, 0.879_092_2];

const OUT_R: [f32; 3] = [1.196_799_4, -0.052_834_3, -0.143_965];
const OUT_G: [f32; 3] = [-0.058_063_2, 1.151_812_1, -0.093_748_94];
const OUT_B: [f32; 3] = [-0.086_345_55, -0.082_385_42, 1.168_730_9];

fn mat_mul(c: [f32; 3], r: [f32; 3], g: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        c[0] * r[0] + c[1] * r[1] + c[2] * r[2],
        c[0] * g[0] + c[1] * g[1] + c[2] * g[2],
        c[0] * b[0] + c[1] * b[1] + c[2] * b[2],
    ]
}

fn log2_normalize(v: f32) -> f32 {
    let clamped = v.max(MIN_LIN);
    ((clamped.log2() - MIN_EV) / RANGE_EV).clamp(0.0, 1.0)
}

fn sigmoid(x: f32) -> f32 {
    let x2 = x * x;
    let x4 = x2 * x2;
    let x6 = x4 * x2;
    -17.866_29 * x6 * x
        + 78.011_72 * x6
        + -126.701_2 * x4 * x
        + 92.060_05 * x4
        + -28.722_15 * x2 * x
        + 4.361_57 * x2
        + 0.139_142_5 * x
        + 0.001_891_437_3
}

fn highlight_desat(rgb: [f32; 3]) -> [f32; 3] {
    let m = rgb[0].max(rgb[1]).max(rgb[2]);
    if m <= DESAT_KNEE {
        return rgb;
    }
    let y = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
    let t_lin = ((m - DESAT_KNEE) / (DESAT_CEIL - DESAT_KNEE)).clamp(0.0, 1.0);
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
    let working = mat_mul(compressed, IN_R, IN_G, IN_B);
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
    let display_linear = mat_mul(sigmoided, OUT_R, OUT_G, OUT_B);
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
