fn op_hue_dist(a: f32, b: f32) -> f32 {
    let raw = a - b;
    let wrapped = raw - floor(raw / 360.0) * 360.0;
    return min(wrapped, 360.0 - wrapped);
}
