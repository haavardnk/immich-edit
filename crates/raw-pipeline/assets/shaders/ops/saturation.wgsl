fn saturation_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> {
    if (p.x == 0.0) { return c; }
    let f = 1.0 + p.x;
    let luma = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    return vec3<f32>(luma) + (c - vec3<f32>(luma)) * f;
}
