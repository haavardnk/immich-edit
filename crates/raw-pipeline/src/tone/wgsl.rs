pub const TONE_WGSL: &str = r#"
const TONE_KIND_DEFAULT: u32 = 0u;
const TONE_KIND_AGX: u32 = 1u;

const AGX_MIN_EV: f32 = -10.0;
const AGX_MAX_EV: f32 = 6.5;
const AGX_MIN_LIN: f32 = 0.0009765625;
const AGX_DESAT_KNEE: f32 = 0.7;
const AGX_DESAT_CEIL: f32 = 2.0;

const AGX_IN_R: vec3<f32> = vec3<f32>(0.8424790, 0.0784110, 0.0878288);
const AGX_IN_G: vec3<f32> = vec3<f32>(0.0423290, 0.8788692, 0.0788438);
const AGX_IN_B: vec3<f32> = vec3<f32>(0.0424243, 0.0788438, 0.8790922);

const AGX_OUT_R: vec3<f32> = vec3<f32>(1.1967994, -0.0528343, -0.1439650);
const AGX_OUT_G: vec3<f32> = vec3<f32>(-0.0580632, 1.1518121, -0.0937489);
const AGX_OUT_B: vec3<f32> = vec3<f32>(-0.0863455, -0.0823854, 1.1687309);

fn tone_soft_clip_high(v: f32) -> f32 {
    let knee: f32 = 0.95;
    if (v <= knee) { return v; }
    let headroom: f32 = 1.0 - knee;
    let excess: f32 = v - knee;
    return knee + headroom * (excess / (excess + headroom));
}

fn tone_srgb_oetf(v: f32) -> f32 {
    var lin = v;
    if (lin <= 0.003130808) {
        return 12.92 * lin;
    }
    return 1.055 * pow(lin, 1.0 / 2.4) - 0.055;
}

fn tone_default_scalar(v: f32) -> f32 {
    var lin: f32;
    if (v <= 0.0) { lin = 0.0; } else { lin = tone_soft_clip_high(v); }
    let srgb = tone_srgb_oetf(lin);
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    return srgb + (s - srgb) * 0.15;
}

fn tone_default_rgb(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(tone_default_scalar(c.x), tone_default_scalar(c.y), tone_default_scalar(c.z));
}

fn agx_log2_normalize(v: f32) -> f32 {
    let clamped = max(v, AGX_MIN_LIN);
    let t = (log2(clamped) - AGX_MIN_EV) / (AGX_MAX_EV - AGX_MIN_EV);
    return clamp(t, 0.0, 1.0);
}

fn agx_sigmoid(x: f32) -> f32 {
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x2 * x2;
    let x5 = x4 * x;
    let x6 = x4 * x2;
    let x7 = x6 * x;
    return -17.86629 * x7
        + 78.01172 * x6
        + -126.7012 * x5
        + 92.06005 * x4
        + -28.72215 * x3
        + 4.36157 * x2
        + 0.1391425 * x
        + 0.0018914373;
}

fn agx_highlight_desat(c: vec3<f32>) -> vec3<f32> {
    let m = max(c.x, max(c.y, c.z));
    if (m <= AGX_DESAT_KNEE) { return c; }
    let y = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
    let t_lin = clamp((m - AGX_DESAT_KNEE) / (AGX_DESAT_CEIL - AGX_DESAT_KNEE), 0.0, 1.0);
    let t = t_lin * t_lin * (3.0 - 2.0 * t_lin);
    let k = 1.0 - t;
    return vec3<f32>(y) + (c - vec3<f32>(y)) * k;
}

fn tone_agx_rgb(c: vec3<f32>) -> vec3<f32> {
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
}

fn tone_apply_rgb(c: vec3<f32>, kind: u32) -> vec3<f32> {
    if (kind == TONE_KIND_AGX) {
        return tone_agx_rgb(c);
    }
    return tone_default_rgb(c);
}
"#;
