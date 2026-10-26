fn lens_mask_to_scene_uv(lens: vec4<f32>, uv: vec2<f32>, dims: vec2<f32>) -> vec2<f32> {
    let k1 = lens.x;
    let k2 = lens.y;
    let k3 = lens.z;
    let zoom = lens.w;
    if (k1 == 0.0 && k2 == 0.0 && k3 == 0.0 && zoom == 1.0) {
        return uv;
    }
    let half_diag = 0.5 * sqrt(dims.x * dims.x + dims.y * dims.y);
    let nx = (uv.x - 0.5) * dims.x;
    let ny = (uv.y - 0.5) * dims.y;
    let r = sqrt(nx * nx + ny * ny) * zoom / max(half_diag, 1e-6);
    let r2 = r * r;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let s = 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
    return vec2<f32>(0.5 + (uv.x - 0.5) * zoom * s, 0.5 + (uv.y - 0.5) * zoom * s);
}
