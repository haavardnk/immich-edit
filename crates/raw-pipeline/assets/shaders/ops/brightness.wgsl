fn brightness_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> {
    let a = clamp(p.x, -1.0, 1.0);
    let y0 = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
    if (y0 <= 1e-5) { return c; }
    let yc = clamp(y0, 0.0, 1.0);
    let d = yc + (1.0 - yc) * exp2(-a * 1.5);
    var yl: f32;
    if (d > 1e-5) { yl = yc / d; } else { yl = yc; }
    let guard = max(y0, max(c.x, max(c.y, c.z)));
    let rolloff = smoothstep(0.9, 1.0, guard);
    let y1 = yl * (1.0 - rolloff) + y0 * rolloff;
    let s = clamp(y1 / y0, 0.0, 8.0);
    return c * s;
}
