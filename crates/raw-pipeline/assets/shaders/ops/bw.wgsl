fn bw_hue(c: vec3<f32>, mx: f32, d: f32) -> f32 {
    var h: f32;
    if (mx == c.r) {
        h = (c.g - c.b) / d + select(0.0, 6.0, c.g < c.b);
    } else if (mx == c.g) {
        h = (c.b - c.r) / d + 2.0;
    } else {
        h = (c.r - c.g) / d + 4.0;
    }
    return h * 60.0;
}

fn bw_channel_mix(hue: f32) -> f32 {
    var mix = array<f32, 6>(p.bw[0].x, p.bw[0].y, p.bw[0].z, p.bw[0].w, p.bw[1].x, p.bw[1].y);
    let h = (hue - floor(hue / 360.0) * 360.0) / 60.0;
    let i = min(u32(floor(h)), 5u);
    let t = h - f32(i);
    return mix[i] * (1.0 - t) + mix[(i + 1u) % 6u] * t;
}

fn bw_apply(c_in: vec3<f32>) -> vec3<f32> {
    let c = max(c_in, vec3<f32>(0.0));
    let y = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    let mx = max(c.r, max(c.g, c.b));
    let d = mx - min(c.r, min(c.g, c.b));
    var weight = 0.0;
    if (d > 1e-6 && mx > 1e-6) {
        weight = d / mx * bw_channel_mix(bw_hue(c, mx, d));
    }
    let gray = y * exp2(BW_MIX_EV * weight);
    let e = sqrt(clamp(gray, 0.0, 1.0));
    let pivot = BW_TINT_PIVOT - BW_TINT_BALANCE_RANGE * p.bw[2].w;
    let wh = smoothstep(pivot - BW_TINT_WIDTH, pivot + BW_TINT_WIDTH, e);
    let tint = p.bw[2].xyz * (1.0 - wh) + p.bw[3].xyz * wh;
    return max(vec3<f32>(gray) * (vec3<f32>(1.0) + tint), vec3<f32>(0.0));
}
