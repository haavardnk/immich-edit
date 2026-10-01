fn contrast_bias(x: f32, k: f32) -> f32 {
    return x / (k * (1.0 - x) + 1.0);
}

fn contrast_one(v: f32, s: f32) -> f32 {
    if (v >= 1.0) { return v; }
    if (v <= 0.0) { return v * pow(s, -CONTRAST_GAMMA); }
    let g = pow(v, 1.0 / CONTRAST_GAMMA);
    let k = s - 1.0;
    var out_v: f32;
    if (g < 0.5) {
        out_v = 0.5 * contrast_bias(2.0 * g, k);
    } else {
        out_v = 1.0 - 0.5 * contrast_bias(2.0 - 2.0 * g, k);
    }
    return pow(out_v, CONTRAST_GAMMA);
}

fn contrast_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> {
    if (p.x == 1.0) { return c; }
    let lo = min(min(c.x, c.y), c.z);
    let hi = max(max(c.x, c.y), c.z);
    return op_rgb_tone(c, contrast_one(lo, p.x), contrast_one(hi, p.x));
}
