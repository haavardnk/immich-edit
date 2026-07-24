use std::sync::LazyLock;

use super::shared::{
    LUMA_B, LUMA_G, LUMA_R, S_CURVE_BLEND, SRGB_OETF_GAMMA, SRGB_OETF_GAMMA_OFFSET,
    SRGB_OETF_GAMMA_SCALE, SRGB_OETF_LINEAR_CUTOFF, SRGB_OETF_LINEAR_SLOPE, TONE_SHOULDER_KNEE,
};

static TONE_WGSL_STR: LazyLock<String> = LazyLock::new(|| {
    format!(
        r#"
fn tone_highlight_shoulder(x: f32) -> f32 {{
    let knee: f32 = {shoulder_knee};
    if (x <= knee) {{ return x; }}
    let headroom: f32 = 1.0 - knee;
    return 1.0 - headroom * exp(-(x - knee) / headroom);
}}

fn tone_srgb_oetf(v: f32) -> f32 {{
    var lin = v;
    if (lin <= {srgb_cutoff}) {{
        return {srgb_slope} * lin;
    }}
    return {srgb_scale} * pow(lin, {srgb_gamma}) - {srgb_offset};
}}

fn tone_display_encode(v: f32) -> f32 {{
    let c = clamp(v, 0.0, 1.0);
    let srgb = tone_srgb_oetf(c);
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    return srgb + (s - srgb) * {s_curve_blend};
}}

fn tone_default_scalar(v: f32) -> f32 {{
    return tone_display_encode(tone_highlight_shoulder(max(v, 0.0)));
}}

fn tone_luma(c: vec3<f32>) -> f32 {{
    return {luma_r} * c.x + {luma_g} * c.y + {luma_b} * c.z;
}}

fn tone_project_gamut(c: vec3<f32>, neutral: f32) -> vec3<f32> {{
    var out = c;
    let mn = min(out.x, min(out.y, out.z));
    if (mn < 0.0) {{
        let t = clamp(-mn / (neutral - mn), 0.0, 1.0);
        out = out + (vec3<f32>(neutral) - out) * t;
    }}
    let mx = max(out.x, max(out.y, out.z));
    if (mx > 1.0) {{
        let t = clamp((mx - 1.0) / (mx - neutral), 0.0, 1.0);
        out = out + (vec3<f32>(neutral) - out) * t;
    }}
    return out;
}}

fn tone_default_rgb(c: vec3<f32>) -> vec3<f32> {{
    let y = tone_luma(c);
    if (y <= 1e-6) {{ return vec3<f32>(0.0, 0.0, 0.0); }}
    let yd = tone_highlight_shoulder(y);
    let mapped = tone_project_gamut(c * (yd / y), yd);
    return vec3<f32>(
        tone_display_encode(mapped.x),
        tone_display_encode(mapped.y),
        tone_display_encode(mapped.z),
    );
}}

fn tone_dither_hash(x: u32, y: u32, c: u32) -> f32 {{
    var h: u32 = x * 0x8da6b343u ^ y * 0xd8163841u ^ c * 0xcb1ab31fu;
    h ^= h >> 16u;
    h = h * 0x7feb352du;
    h ^= h >> 15u;
    h = h * 0x846ca68bu;
    h ^= h >> 16u;
    return f32(h) / f32(0xffffffffu);
}}

fn tone_dither_u8(c: vec3<f32>, x: u32, y: u32) -> vec3<f32> {{
    let dr = (tone_dither_hash(x, y, 0u) - tone_dither_hash(x, y, 1u)) / 255.0;
    let dg = (tone_dither_hash(x, y, 2u) - tone_dither_hash(x, y, 3u)) / 255.0;
    let db = (tone_dither_hash(x, y, 4u) - tone_dither_hash(x, y, 5u)) / 255.0;
    return clamp(c + vec3<f32>(dr, dg, db), vec3<f32>(0.0), vec3<f32>(1.0));
}}

fn tone_dcp_rgb(c: vec3<f32>) -> vec3<f32> {{
    let neutral = clamp(tone_luma(c), 0.0, 1.0);
    let mapped = tone_project_gamut(c, neutral);
    return vec3<f32>(
        tone_srgb_oetf(clamp(mapped.x, 0.0, 1.0)),
        tone_srgb_oetf(clamp(mapped.y, 0.0, 1.0)),
        tone_srgb_oetf(clamp(mapped.z, 0.0, 1.0)),
    );
}}

fn tone_apply_rgb(c: vec3<f32>) -> vec3<f32> {{
    return tone_default_rgb(c);
}}

fn tone_to_output_space(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    if (p3 == 0u) {{ return c; }}
    return vec3<f32>(
        {m00} * c.x + {m01} * c.y + {m02} * c.z,
        {m10} * c.x + {m11} * c.y + {m12} * c.z,
        {m20} * c.x + {m21} * c.y + {m22} * c.z,
    );
}}

fn tone_default_rgb_cs(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    let y = tone_luma(c);
    if (y <= 1e-6) {{ return vec3<f32>(0.0, 0.0, 0.0); }}
    let yd = tone_highlight_shoulder(y);
    let mapped = tone_project_gamut(tone_to_output_space(c * (yd / y), p3), yd);
    return vec3<f32>(
        tone_display_encode(mapped.x),
        tone_display_encode(mapped.y),
        tone_display_encode(mapped.z),
    );
}}

fn tone_dcp_rgb_cs(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    let neutral = clamp(tone_luma(c), 0.0, 1.0);
    let mapped = tone_project_gamut(tone_to_output_space(c, p3), neutral);
    return vec3<f32>(
        tone_srgb_oetf(clamp(mapped.x, 0.0, 1.0)),
        tone_srgb_oetf(clamp(mapped.y, 0.0, 1.0)),
        tone_srgb_oetf(clamp(mapped.z, 0.0, 1.0)),
    );
}}
"#,
        shoulder_knee = TONE_SHOULDER_KNEE,
        srgb_cutoff = SRGB_OETF_LINEAR_CUTOFF,
        srgb_slope = SRGB_OETF_LINEAR_SLOPE,
        srgb_scale = SRGB_OETF_GAMMA_SCALE,
        srgb_gamma = SRGB_OETF_GAMMA,
        srgb_offset = SRGB_OETF_GAMMA_OFFSET,
        s_curve_blend = S_CURVE_BLEND,
        luma_r = LUMA_R,
        luma_g = LUMA_G,
        luma_b = LUMA_B,
        m00 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[0][0]),
        m01 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[0][1]),
        m02 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[0][2]),
        m10 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[1][0]),
        m11 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[1][1]),
        m12 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[1][2]),
        m20 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[2][0]),
        m21 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[2][1]),
        m22 = wgsl_f32(crate::color::SRGB_LINEAR_TO_DISPLAY_P3[2][2]),
    )
});

fn wgsl_f32(v: f32) -> String {
    let s = format!("{v:?}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

pub fn tone_wgsl() -> &'static str {
    &TONE_WGSL_STR
}
