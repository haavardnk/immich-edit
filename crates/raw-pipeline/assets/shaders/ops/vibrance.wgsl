fn vibrance_apply(c: vec3<f32>, amount: f32) -> vec3<f32> {
    if (amount == 0.0) { return c; }
    let mx = max(max(c.r, c.g), c.b);
    let mn = min(min(c.r, c.g), c.b);
    let d = mx - mn;
    let chroma = clamp(d, 0.0, 1.0);
    var hue: f32 = 0.0;
    if (d >= 1e-6) {
        if (mx == c.r) {
            var k = (c.g - c.b) / d;
            if (c.g < c.b) { k = k + 6.0; }
            hue = k * 60.0;
        } else if (mx == c.g) {
            hue = ((c.b - c.r) / d + 2.0) * 60.0;
        } else {
            hue = ((c.r - c.g) / d + 4.0) * 60.0;
        }
    }
    var effective: f32;
    if (amount > 0.0) {
        let base = amount * VIBRANCE_GAIN * (1.0 - smoothstep(VIBRANCE_CHROMA_LO, VIBRANCE_CHROMA_HI, chroma));
        var skin = 1.0 - smoothstep(VIBRANCE_SKIN_SPREAD_LO_DEG, VIBRANCE_SKIN_SPREAD_HI_DEG, op_hue_dist(hue, VIBRANCE_SKIN_HUE_DEG));
        skin = skin * smoothstep(VIBRANCE_SKIN_CHROMA_LO, VIBRANCE_SKIN_CHROMA_HI, chroma);
        effective = base * (1.0 + (VIBRANCE_SKIN_FACTOR - 1.0) * skin);
    } else {
        effective = amount * (1.0 - smoothstep(VIBRANCE_DESAT_LO, VIBRANCE_DESAT_HI, chroma));
    }
    if (abs(effective) < 1e-5) { return c; }
    let factor = 1.0 + effective;
    let luma = 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    return vec3<f32>(luma) + (c - vec3<f32>(luma)) * factor;
}
