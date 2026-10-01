struct DisplayCurves {{
    enabled: vec4<f32>,
    lut: array<vec4<f32>, {vec4s}>,
}};

@group(0) @binding({binding}) var<uniform> display_curves: DisplayCurves;

const DISPLAY_CURVES_SIZE: u32 = {size}u;

fn display_curves_get(base: u32, idx: u32) -> f32 {{
    let i = base + idx;
    return display_curves.lut[i / 4u][i % 4u];
}}

fn display_curves_sample(base: u32, x: f32) -> f32 {{
    let last = DISPLAY_CURVES_SIZE - 1u;
    let cx = clamp(x, 0.0, 1.0) * f32(last);
    let idx = u32(cx);
    let frac = cx - f32(idx);
    let v0 = display_curves_get(base, idx);
    let v1 = display_curves_get(base, min(idx + 1u, last));
    return mix(v0, v1, frac);
}}

fn display_curves_apply(c: vec3<f32>) -> vec3<f32> {{
    if (display_curves.enabled.x < 0.5) {{
        return c;
    }}
    let n = DISPLAY_CURVES_SIZE;
    let r = display_curves_sample(n, display_curves_sample(0u, c.r));
    let g = display_curves_sample(2u * n, display_curves_sample(0u, c.g));
    let b = display_curves_sample(3u * n, display_curves_sample(0u, c.b));
    let y0 = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let y1 = display_curves_sample(4u * n, y0);
    if (y0 < 1e-5) {{
        return vec3<f32>(y1);
    }}
    let scaled = vec3<f32>(r, g, b) * (y1 / y0);
    let mx = max(scaled.r, max(scaled.g, scaled.b));
    if (mx <= 1.0) {{
        return scaled;
    }}
    return mix(scaled, vec3<f32>(y1), (mx - 1.0) / (mx - y1));
}}
