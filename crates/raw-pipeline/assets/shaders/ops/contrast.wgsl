fn contrast_one(v: f32, s: f32) -> f32 {
    let g = pow(max(v, 0.0), 1.0 / 2.2);
    let gc = clamp(g, 0.0, 1.0);
    var out_v: f32;
    if (gc < 0.5) {
        out_v = 0.5 * pow(2.0 * gc, s);
    } else {
        out_v = 1.0 - 0.5 * pow(2.0 * (1.0 - gc), s);
    }
    let lin = pow(out_v, 2.2);
    let m = smoothstep(1.0, 1.01, v);
    return lin * (1.0 - m) + v * m;
}

fn contrast_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> {
    if (p.x == 1.0) { return c; }
    return vec3<f32>(contrast_one(c.x, p.x), contrast_one(c.y, p.x), contrast_one(c.z, p.x));
}
