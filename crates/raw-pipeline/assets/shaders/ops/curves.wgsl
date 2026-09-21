const CURVES_SIZE: u32 = {size}u;

fn curves_get(base: u32, idx: u32) -> f32 {{
    let abs_idx = base + idx;
    let vi = abs_idx / 4u;
    let ci = abs_idx % 4u;
    return p.curves[vi][ci];
}}

fn curves_sample(base: u32, x: f32) -> f32 {{
    let last = CURVES_SIZE - 1u;
    let cx = clamp(x, 0.0, 1.0) * f32(last);
    let idx = u32(cx);
    let frac = cx - f32(idx);
    let v0 = curves_get(base, idx);
    let v1 = curves_get(base, min(idx + 1u, last));
    return mix(v0, v1, frac);
}}

fn curves_apply(c: vec3<f32>) -> vec3<f32> {{
    var r = curves_sample(0u, c.x);
    var g = curves_sample(0u, c.y);
    var b = curves_sample(0u, c.z);
    r = curves_sample(CURVES_SIZE, r);
    g = curves_sample(2u * CURVES_SIZE, g);
    b = curves_sample(3u * CURVES_SIZE, b);
    let y0 = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let y0c = clamp(y0, 0.0, 1.0);
    let y1 = curves_sample(4u * CURVES_SIZE, y0c);
    if (y0 < 1e-5) {{
        return vec3<f32>(y1);
    }}
    let scale = y1 / y0;
    return vec3<f32>(r * scale, g * scale, b * scale);
}}
