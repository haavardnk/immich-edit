use std::sync::LazyLock;

use super::shared::{
    AGX_DESAT_CEIL, AGX_DESAT_KNEE, AGX_IN_B, AGX_IN_G, AGX_IN_R, AGX_MAX_EV, AGX_MIN_EV,
    AGX_MIN_LIN, AGX_OUT_B, AGX_OUT_G, AGX_OUT_R, AGX_SIGMOID_COEFFS, HIGHLIGHT_KNEE,
    S_CURVE_BLEND, SRGB_OETF_GAMMA, SRGB_OETF_GAMMA_OFFSET, SRGB_OETF_GAMMA_SCALE,
    SRGB_OETF_LINEAR_CUTOFF, SRGB_OETF_LINEAR_SLOPE, TONE_KIND_AGX, TONE_KIND_DEFAULT,
};

fn vec3(v: [f32; 3]) -> String {
    format!("vec3<f32>({}, {}, {})", v[0], v[1], v[2])
}

static TONE_WGSL_STR: LazyLock<String> = LazyLock::new(|| {
    let c = AGX_SIGMOID_COEFFS;
    format!(
        r#"
const TONE_KIND_DEFAULT: u32 = {tone_default}u;
const TONE_KIND_AGX: u32 = {tone_agx}u;

const AGX_MIN_EV: f32 = {agx_min_ev};
const AGX_MAX_EV: f32 = {agx_max_ev};
const AGX_MIN_LIN: f32 = {agx_min_lin};
const AGX_DESAT_KNEE: f32 = {agx_desat_knee};
const AGX_DESAT_CEIL: f32 = {agx_desat_ceil};

const AGX_IN_R: vec3<f32> = {agx_in_r};
const AGX_IN_G: vec3<f32> = {agx_in_g};
const AGX_IN_B: vec3<f32> = {agx_in_b};

const AGX_OUT_R: vec3<f32> = {agx_out_r};
const AGX_OUT_G: vec3<f32> = {agx_out_g};
const AGX_OUT_B: vec3<f32> = {agx_out_b};

fn tone_soft_clip_high(v: f32) -> f32 {{
    let knee: f32 = {highlight_knee};
    if (v <= knee) {{ return v; }}
    let headroom: f32 = 1.0 - knee;
    let excess: f32 = v - knee;
    return knee + headroom * (excess / (excess + headroom));
}}

fn tone_srgb_oetf(v: f32) -> f32 {{
    var lin = v;
    if (lin <= {srgb_cutoff}) {{
        return {srgb_slope} * lin;
    }}
    return {srgb_scale} * pow(lin, {srgb_gamma}) - {srgb_offset};
}}

fn tone_default_scalar(v: f32) -> f32 {{
    var lin: f32;
    if (v <= 0.0) {{ lin = 0.0; }} else {{ lin = tone_soft_clip_high(v); }}
    let srgb = tone_srgb_oetf(lin);
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    return srgb + (s - srgb) * {s_curve_blend};
}}

fn tone_default_rgb(c: vec3<f32>) -> vec3<f32> {{
    return vec3<f32>(tone_default_scalar(c.x), tone_default_scalar(c.y), tone_default_scalar(c.z));
}}

fn agx_log2_normalize(v: f32) -> f32 {{
    let clamped = max(v, AGX_MIN_LIN);
    let t = (log2(clamped) - AGX_MIN_EV) / (AGX_MAX_EV - AGX_MIN_EV);
    return clamp(t, 0.0, 1.0);
}}

fn agx_sigmoid(x: f32) -> f32 {{
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x2 * x2;
    let x5 = x4 * x;
    let x6 = x4 * x2;
    let x7 = x6 * x;
    return {c0} * x7
        + {c1} * x6
        + {c2} * x5
        + {c3} * x4
        + {c4} * x3
        + {c5} * x2
        + {c6} * x
        + {c7};
}}

fn agx_highlight_desat(c: vec3<f32>) -> vec3<f32> {{
    let m = max(c.x, max(c.y, c.z));
    if (m <= AGX_DESAT_KNEE) {{ return c; }}
    let y = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
    let t_lin = clamp((m - AGX_DESAT_KNEE) / (AGX_DESAT_CEIL - AGX_DESAT_KNEE), 0.0, 1.0);
    let t = t_lin * t_lin * (3.0 - 2.0 * t_lin);
    let k = 1.0 - t;
    return vec3<f32>(y) + (c - vec3<f32>(y)) * k;
}}

fn tone_agx_rgb(c: vec3<f32>) -> vec3<f32> {{
    let comp = agx_highlight_desat(c);
    let working = vec3<f32>(
        dot(comp, AGX_IN_R),
        dot(comp, AGX_IN_G),
        dot(comp, AGX_IN_B),
    );
    let log_rgb = vec3<f32>(
        agx_log2_normalize(working.x),
        agx_log2_normalize(working.y),
        agx_log2_normalize(working.z),
    );
    let sig = vec3<f32>(
        agx_sigmoid(log_rgb.x),
        agx_sigmoid(log_rgb.y),
        agx_sigmoid(log_rgb.z),
    );
    let display = vec3<f32>(
        dot(sig, AGX_OUT_R),
        dot(sig, AGX_OUT_G),
        dot(sig, AGX_OUT_B),
    );
    return clamp(display, vec3<f32>(0.0), vec3<f32>(1.0));
}}

fn tone_apply_rgb(c: vec3<f32>, kind: u32) -> vec3<f32> {{
    if (kind == TONE_KIND_AGX) {{
        return tone_agx_rgb(c);
    }}
    return tone_default_rgb(c);
}}
"#,
        tone_default = TONE_KIND_DEFAULT,
        tone_agx = TONE_KIND_AGX,
        agx_min_ev = AGX_MIN_EV,
        agx_max_ev = AGX_MAX_EV,
        agx_min_lin = AGX_MIN_LIN,
        agx_desat_knee = AGX_DESAT_KNEE,
        agx_desat_ceil = AGX_DESAT_CEIL,
        agx_in_r = vec3(AGX_IN_R),
        agx_in_g = vec3(AGX_IN_G),
        agx_in_b = vec3(AGX_IN_B),
        agx_out_r = vec3(AGX_OUT_R),
        agx_out_g = vec3(AGX_OUT_G),
        agx_out_b = vec3(AGX_OUT_B),
        highlight_knee = HIGHLIGHT_KNEE,
        srgb_cutoff = SRGB_OETF_LINEAR_CUTOFF,
        srgb_slope = SRGB_OETF_LINEAR_SLOPE,
        srgb_scale = SRGB_OETF_GAMMA_SCALE,
        srgb_gamma = SRGB_OETF_GAMMA,
        srgb_offset = SRGB_OETF_GAMMA_OFFSET,
        s_curve_blend = S_CURVE_BLEND,
        c0 = c[0],
        c1 = c[1],
        c2 = c[2],
        c3 = c[3],
        c4 = c[4],
        c5 = c[5],
        c6 = c[6],
        c7 = c[7],
    )
});

pub fn tone_wgsl() -> &'static str {
    &TONE_WGSL_STR
}
