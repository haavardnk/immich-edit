fn white_balance_apply(c: vec3<f32>, w: vec4<f32>) -> vec3<f32> {
    return c * w.rgb;
}
