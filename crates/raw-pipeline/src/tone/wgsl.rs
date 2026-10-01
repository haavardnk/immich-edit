use std::sync::LazyLock;

use super::gamut::{GAMUT_SEARCH_STEPS, OKLAB_L_WEIGHTS, lms_basis};
use super::shared::{
    SRGB_OETF_GAMMA, SRGB_OETF_GAMMA_OFFSET, SRGB_OETF_GAMMA_SCALE, SRGB_OETF_LINEAR_CUTOFF,
    SRGB_OETF_LINEAR_SLOPE,
};
use crate::frame::OutputColorSpace;
use crate::ops::wgsl::f32_lit;

fn vec3_lit(v: [f32; 3]) -> String {
    format!(
        "vec3<f32>({}, {}, {})",
        f32_lit(v[0]),
        f32_lit(v[1]),
        f32_lit(v[2])
    )
}

fn mat_apply(m: &[[f32; 3]; 3], v: &str) -> String {
    format!(
        "vec3<f32>(dot({}, {v}), dot({}, {v}), dot({}, {v}))",
        vec3_lit(m[0]),
        vec3_lit(m[1]),
        vec3_lit(m[2])
    )
}

static TONE_WGSL_STR: LazyLock<String> = LazyLock::new(|| {
    let srgb = lms_basis(OutputColorSpace::SRgb);
    let p3 = lms_basis(OutputColorSpace::DisplayP3);
    format!(
        r#"
fn tone_srgb_oetf(v: f32) -> f32 {{
    var lin = v;
    if (lin <= {srgb_cutoff}) {{
        return {srgb_slope} * lin;
    }}
    return {srgb_scale} * pow(lin, {srgb_gamma}) - {srgb_offset};
}}

fn tone_in_unit_cube(c: vec3<f32>) -> bool {{
    return all(c >= vec3<f32>(0.0)) && all(c <= vec3<f32>(1.0));
}}

fn tone_to_lms(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    if (p3 == 0u) {{ return {srgb_to_lms}; }}
    return {p3_to_lms};
}}

fn tone_from_lms(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    if (p3 == 0u) {{ return {srgb_from_lms}; }}
    return {p3_from_lms};
}}

fn tone_gamut_at(lms: vec3<f32>, l0: f32, t: f32, p3: u32) -> vec3<f32> {{
    let x = vec3<f32>(l0) + t * (lms - vec3<f32>(l0));
    return tone_from_lms(x * x * x, p3);
}}

fn tone_map_to_gamut(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    if (tone_in_unit_cube(c)) {{ return c; }}
    let lin_lms = tone_to_lms(c, p3);
    let lms = sign(lin_lms) * pow(abs(lin_lms), vec3<f32>(1.0 / 3.0));
    let l0 = clamp(dot({oklab_l}, lms), 0.0, 1.0);
    var lo = 0.0;
    var hi = 1.0;
    for (var i = 0u; i < {gamut_steps}u; i = i + 1u) {{
        let mid = 0.5 * (lo + hi);
        if (tone_in_unit_cube(tone_gamut_at(lms, l0, mid, p3))) {{
            lo = mid;
        }} else {{
            hi = mid;
        }}
    }}
    return tone_gamut_at(lms, l0, lo, p3);
}}

fn tone_dither_hash(x: u32, y: u32, c: u32) -> f32 {{
    var h: u32 = (x * 0x8da6b343u) ^ (y * 0xd8163841u) ^ (c * 0xcb1ab31fu);
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

fn tone_to_output_space(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    if (p3 == 0u) {{ return c; }}
    return {srgb_to_p3};
}}

fn tone_rgb_cs(c: vec3<f32>, p3: u32) -> vec3<f32> {{
    let mapped = tone_map_to_gamut(tone_to_output_space(c, p3), p3);
    return vec3<f32>(
        tone_srgb_oetf(clamp(mapped.x, 0.0, 1.0)),
        tone_srgb_oetf(clamp(mapped.y, 0.0, 1.0)),
        tone_srgb_oetf(clamp(mapped.z, 0.0, 1.0)),
    );
}}

fn tone_apply_rgb(c: vec3<f32>) -> vec3<f32> {{
    return tone_rgb_cs(c, 0u);
}}

fn tone_below_gamut(c: vec3<f32>) -> bool {{
    return min(c.x, min(c.y, c.z)) < -1e-4;
}}

fn tone_is_out_of_gamut(c: vec3<f32>, p3: u32) -> bool {{
    return tone_below_gamut(tone_to_output_space(c, p3));
}}

fn warn_clip_alpha(c: vec3<f32>) -> f32 {{
    if (any(c >= vec3<f32>({clip_high}))) {{ return {alpha_highlight}; }}
    if (any(c <= vec3<f32>({clip_low}))) {{ return {alpha_shadow}; }}
    return 1.0;
}}
"#,
        alpha_highlight = crate::warn::ALPHA_HIGHLIGHT as f32 / 255.0,
        alpha_shadow = crate::warn::ALPHA_SHADOW as f32 / 255.0,
        clip_high = crate::warn::HIGHLIGHT_CLIP,
        clip_low = crate::warn::SHADOW_CLIP,
        srgb_cutoff = SRGB_OETF_LINEAR_CUTOFF,
        srgb_slope = SRGB_OETF_LINEAR_SLOPE,
        srgb_scale = SRGB_OETF_GAMMA_SCALE,
        srgb_gamma = SRGB_OETF_GAMMA,
        srgb_offset = SRGB_OETF_GAMMA_OFFSET,
        srgb_to_p3 = mat_apply(&crate::color::SRGB_LINEAR_TO_DISPLAY_P3, "c"),
        srgb_to_lms = mat_apply(&srgb.to_lms, "c"),
        p3_to_lms = mat_apply(&p3.to_lms, "c"),
        srgb_from_lms = mat_apply(&srgb.from_lms, "c"),
        p3_from_lms = mat_apply(&p3.from_lms, "c"),
        oklab_l = vec3_lit(OKLAB_L_WEIGHTS),
        gamut_steps = GAMUT_SEARCH_STEPS,
    )
});

pub fn tone_wgsl() -> &'static str {
    &TONE_WGSL_STR
}
