fn saturation_apply(c: vec3<f32>, p: vec4<f32>) -> vec3<f32> {
    if (p.x == 0.0) { return c; }
    let lab = op_to_oklab(c);
    return op_from_oklab(vec3<f32>(lab.x, lab.yz * (1.0 + p.x)));
}
