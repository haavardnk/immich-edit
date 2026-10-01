use std::fmt::Write;
use std::sync::LazyLock;

use super::brightness::{
    BRIGHTNESS_K, BRIGHTNESS_MAX_GAIN, BRIGHTNESS_ROLLOFF_HI, BRIGHTNESS_ROLLOFF_LO,
};
use super::bw::{BW_MIX_EV, BW_TINT_BALANCE_RANGE, BW_TINT_PIVOT, BW_TINT_WIDTH};
use super::color_grade::{
    COLOR_GRADE_FEATHER_BASE, COLOR_GRADE_FEATHER_RANGE, COLOR_GRADE_PIVOT_BASE,
    COLOR_GRADE_PIVOT_RANGE,
};
use super::contrast::CONTRAST_GAMMA;
use super::hsl::{
    HSL_BAND_CENTERS_DEG, HSL_BAND_SIGMA_DEG, HSL_HUE_SHIFT_DEG, HSL_LUM_EV, HSL_MIN_SAT,
    HSL_PARAM_FULL_SCALE, HSL_SAT_GATE_HI, HSL_SAT_GATE_LO,
};
use super::tone_regions::{
    TONE_REGIONS_BK_MASK_RANGE, TONE_REGIONS_BK_MULT_MAX, TONE_REGIONS_BK_STRENGTH,
    TONE_REGIONS_HL_DESAT_HI, TONE_REGIONS_HL_DESAT_LO, TONE_REGIONS_HL_MASK_HI,
    TONE_REGIONS_HL_MASK_LO, TONE_REGIONS_HL_MASK_TANH, TONE_REGIONS_HL_STRENGTH,
    TONE_REGIONS_SH_HALO_HI, TONE_REGIONS_SH_HALO_LO, TONE_REGIONS_SH_MASK_RANGE,
    TONE_REGIONS_SH_MULT_MAX, TONE_REGIONS_SH_MULT_MIN, TONE_REGIONS_SH_SOURCE_FLOOR,
    TONE_REGIONS_SH_STRENGTH, TONE_REGIONS_WHITES_PIVOT, TONE_REGIONS_WHITES_STOPS,
};
use super::vibrance::{
    VIBRANCE_DESAT_HI, VIBRANCE_DESAT_LO, VIBRANCE_GAIN, VIBRANCE_SAT_HI, VIBRANCE_SAT_LO,
    VIBRANCE_SKIN_FACTOR, VIBRANCE_SKIN_HUE_DEG, VIBRANCE_SKIN_SAT_HI, VIBRANCE_SKIN_SAT_LO,
    VIBRANCE_SKIN_SPREAD_HI_DEG, VIBRANCE_SKIN_SPREAD_LO_DEG,
};
use crate::edits::HSL_BANDS;

const OP_HELPERS: &str = r#"fn op_hue_dist(a: f32, b: f32) -> f32 {
    let raw = a - b;
    let wrapped = raw - floor(raw / 360.0) * 360.0;
    return min(wrapped, 360.0 - wrapped);
}

fn op_rgb_tone(c: vec3<f32>, lo_out: f32, hi_out: f32) -> vec3<f32> {
    let lo = min(min(c.x, c.y), c.z);
    let hi = max(max(c.x, c.y), c.z);
    if (hi <= lo) { return vec3<f32>(hi_out); }
    let scale = (hi_out - lo_out) / (hi - lo);
    return vec3<f32>(lo_out) + (c - vec3<f32>(lo)) * scale;
}
"#;

const OKLAB: &str = r#"fn op_to_oklab(c: vec3<f32>) -> vec3<f32> {
    let lms = vec3<f32>(
        0.41222146 * c.r + 0.53633255 * c.g + 0.051445995 * c.b,
        0.2119035 * c.r + 0.6806995 * c.g + 0.10739696 * c.b,
        0.08830246 * c.r + 0.28171885 * c.g + 0.6299787 * c.b,
    );
    let cb = sign(lms) * pow(abs(lms), vec3<f32>(1.0 / 3.0));
    return vec3<f32>(
        0.21045426 * cb.x + 0.7936178 * cb.y - 0.004072047 * cb.z,
        1.9779985 * cb.x - 2.4285922 * cb.y + 0.4505937 * cb.z,
        0.025904037 * cb.x + 0.78277177 * cb.y - 0.80867577 * cb.z,
    );
}

fn op_from_oklab(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let m_ = lab.x - 0.105561346 * lab.y - 0.06385417 * lab.z;
    let s_ = lab.x - 0.08948418 * lab.y - 1.2914855 * lab.z;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767417 * l3 - 3.3077116 * m3 + 0.23096994 * s3,
        -1.268438 * l3 + 2.6097574 * m3 - 0.34131938 * s3,
        -0.0041960863 * l3 - 0.7034186 * m3 + 1.7076147 * s3,
    );
}
"#;

pub fn f32_lit(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

fn scalars(out: &mut String, entries: &[(&str, f32)]) {
    for (name, value) in entries {
        writeln!(out, "const {name}: f32 = {};", f32_lit(*value)).unwrap();
    }
}

static OP_PRELUDE_WGSL: LazyLock<String> = LazyLock::new(|| {
    let mut out = String::from(OP_HELPERS);
    out.push_str(OKLAB);
    let centers = HSL_BAND_CENTERS_DEG
        .iter()
        .map(|v| f32_lit(*v))
        .collect::<Vec<String>>()
        .join(", ");
    writeln!(out, "const HSL_BANDS: i32 = {HSL_BANDS};").unwrap();
    writeln!(
        out,
        "const HSL_BAND_CENTERS_DEG: array<f32, {HSL_BANDS}> = array<f32, {HSL_BANDS}>({centers});"
    )
    .unwrap();
    scalars(
        &mut out,
        &[
            ("HSL_BAND_SIGMA_DEG", HSL_BAND_SIGMA_DEG),
            ("HSL_PARAM_FULL_SCALE", HSL_PARAM_FULL_SCALE),
            ("HSL_HUE_SHIFT_DEG", HSL_HUE_SHIFT_DEG),
            ("HSL_LUM_EV", HSL_LUM_EV),
            ("HSL_SAT_GATE_LO", HSL_SAT_GATE_LO),
            ("HSL_SAT_GATE_HI", HSL_SAT_GATE_HI),
            ("HSL_MIN_SAT", HSL_MIN_SAT),
            ("BRIGHTNESS_K", BRIGHTNESS_K),
            ("BRIGHTNESS_ROLLOFF_LO", BRIGHTNESS_ROLLOFF_LO),
            ("BRIGHTNESS_ROLLOFF_HI", BRIGHTNESS_ROLLOFF_HI),
            ("BRIGHTNESS_MAX_GAIN", BRIGHTNESS_MAX_GAIN),
            ("CONTRAST_GAMMA", CONTRAST_GAMMA),
            ("VIBRANCE_GAIN", VIBRANCE_GAIN),
            ("VIBRANCE_SAT_LO", VIBRANCE_SAT_LO),
            ("VIBRANCE_SAT_HI", VIBRANCE_SAT_HI),
            ("VIBRANCE_SKIN_HUE_DEG", VIBRANCE_SKIN_HUE_DEG),
            ("VIBRANCE_SKIN_SPREAD_LO_DEG", VIBRANCE_SKIN_SPREAD_LO_DEG),
            ("VIBRANCE_SKIN_SPREAD_HI_DEG", VIBRANCE_SKIN_SPREAD_HI_DEG),
            ("VIBRANCE_SKIN_SAT_LO", VIBRANCE_SKIN_SAT_LO),
            ("VIBRANCE_SKIN_SAT_HI", VIBRANCE_SKIN_SAT_HI),
            ("VIBRANCE_SKIN_FACTOR", VIBRANCE_SKIN_FACTOR),
            ("VIBRANCE_DESAT_LO", VIBRANCE_DESAT_LO),
            ("VIBRANCE_DESAT_HI", VIBRANCE_DESAT_HI),
            ("TONE_REGIONS_WHITES_STOPS", TONE_REGIONS_WHITES_STOPS),
            ("TONE_REGIONS_WHITES_PIVOT", TONE_REGIONS_WHITES_PIVOT),
            ("TONE_REGIONS_HL_MASK_LO", TONE_REGIONS_HL_MASK_LO),
            ("TONE_REGIONS_HL_MASK_HI", TONE_REGIONS_HL_MASK_HI),
            ("TONE_REGIONS_HL_MASK_TANH", TONE_REGIONS_HL_MASK_TANH),
            ("TONE_REGIONS_HL_STRENGTH", TONE_REGIONS_HL_STRENGTH),
            ("TONE_REGIONS_HL_DESAT_LO", TONE_REGIONS_HL_DESAT_LO),
            ("TONE_REGIONS_HL_DESAT_HI", TONE_REGIONS_HL_DESAT_HI),
            ("TONE_REGIONS_SH_MASK_RANGE", TONE_REGIONS_SH_MASK_RANGE),
            ("TONE_REGIONS_SH_HALO_LO", TONE_REGIONS_SH_HALO_LO),
            ("TONE_REGIONS_SH_HALO_HI", TONE_REGIONS_SH_HALO_HI),
            ("TONE_REGIONS_SH_STRENGTH", TONE_REGIONS_SH_STRENGTH),
            ("TONE_REGIONS_SH_MULT_MIN", TONE_REGIONS_SH_MULT_MIN),
            ("TONE_REGIONS_SH_MULT_MAX", TONE_REGIONS_SH_MULT_MAX),
            ("TONE_REGIONS_SH_SOURCE_FLOOR", TONE_REGIONS_SH_SOURCE_FLOOR),
            ("TONE_REGIONS_BK_MASK_RANGE", TONE_REGIONS_BK_MASK_RANGE),
            ("TONE_REGIONS_BK_STRENGTH", TONE_REGIONS_BK_STRENGTH),
            ("TONE_REGIONS_BK_MULT_MAX", TONE_REGIONS_BK_MULT_MAX),
            ("COLOR_GRADE_PIVOT_BASE", COLOR_GRADE_PIVOT_BASE),
            ("COLOR_GRADE_PIVOT_RANGE", COLOR_GRADE_PIVOT_RANGE),
            ("COLOR_GRADE_FEATHER_BASE", COLOR_GRADE_FEATHER_BASE),
            ("COLOR_GRADE_FEATHER_RANGE", COLOR_GRADE_FEATHER_RANGE),
            ("BW_MIX_EV", BW_MIX_EV),
            ("BW_TINT_PIVOT", BW_TINT_PIVOT),
            ("BW_TINT_BALANCE_RANGE", BW_TINT_BALANCE_RANGE),
            ("BW_TINT_WIDTH", BW_TINT_WIDTH),
        ],
    );
    out
});

pub fn op_prelude_wgsl() -> &'static str {
    &OP_PRELUDE_WGSL
}
