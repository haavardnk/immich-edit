fn load_clamped(ix: i32, iy: i32) -> vec3<f32> {
    let last = vec2<i32>(textureDimensions(src)) - 1;
    let cx = clamp(ix, 0, last.x);
    let cy = clamp(iy, 0, last.y);
    return textureLoad(src, vec2<i32>(cx, cy), 0).rgb;
}

fn cr_weights(t: f32) -> vec4<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    return vec4<f32>(
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    );
}

fn bicubic_row(ix: i32, iy: i32, wx: vec4<f32>) -> vec3<f32> {
    return load_clamped(ix - 1, iy) * wx.x
        + load_clamped(ix, iy) * wx.y
        + load_clamped(ix + 1, iy) * wx.z
        + load_clamped(ix + 2, iy) * wx.w;
}

fn bicubic_rgb(s: vec2<f32>) -> vec3<f32> {
    let f = floor(s);
    let ix = i32(f.x) - p.src_origin.x;
    let iy = i32(f.y) - p.src_origin.y;
    let wx = cr_weights(s.x - f.x);
    let wy = cr_weights(s.y - f.y);
    return bicubic_row(ix, iy - 1, wx) * wy.x
        + bicubic_row(ix, iy, wx) * wy.y
        + bicubic_row(ix, iy + 1, wx) * wy.z
        + bicubic_row(ix, iy + 2, wx) * wy.w;
}
