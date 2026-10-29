fn sigma_level(x: i32, y: i32) -> u32 {
    if (p.levels <= 1u) { return 0u; }
    let px = vec2<u32>(u32(x), u32(y)) + p.origin;
    let d = vec2<f32>(f32(px.x) + 0.5 - 0.5 * f32(p.frame.x), f32(px.y) + 0.5 - 0.5 * f32(p.frame.y));
    let t = sqrt(dot(d, d)) / p.half_diag;
    return min(u32(floor(t * f32(p.levels - 1u) + 0.5)), p.levels - 1u);
}
