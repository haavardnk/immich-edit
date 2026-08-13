fn hsl_rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
    let mx = max(max(c.r, c.g), c.b);
    let mn = min(min(c.r, c.g), c.b);
    let l = (mx + mn) * 0.5;
    let d = mx - mn;
    if (d < 1e-6) { return vec3<f32>(0.0, 0.0, l); }
    var s: f32;
    if (l > 0.5) { s = d / (2.0 - mx - mn); } else { s = d / (mx + mn); }
    var h: f32;
    if (mx == c.r) {
        var k = (c.g - c.b) / d;
        if (c.g < c.b) { k = k + 6.0; }
        h = k;
    } else if (mx == c.g) {
        h = (c.b - c.r) / d + 2.0;
    } else {
        h = (c.r - c.g) / d + 4.0;
    }
    return vec3<f32>(h * 60.0, s, l);
}

fn hsl_hue_to_rgb(p0: f32, q: f32, t_in: f32) -> f32 {
    var t = t_in;
    if (t < 0.0) { t = t + 1.0; }
    if (t > 1.0) { t = t - 1.0; }
    if (t < 1.0 / 6.0) { return p0 + (q - p0) * 6.0 * t; }
    if (t < 0.5) { return q; }
    if (t < 2.0 / 3.0) { return p0 + (q - p0) * (2.0 / 3.0 - t) * 6.0; }
    return p0;
}

fn hsl_hsl_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let h_deg = c.x; let s = c.y; let l = c.z;
    if (s <= 0.0) { return vec3<f32>(l); }
    let h_wrap = h_deg - floor(h_deg / 360.0) * 360.0;
    let h_norm = h_wrap / 360.0;
    var q: f32;
    if (l < 0.5) { q = l * (1.0 + s); } else { q = l + s - l * s; }
    let p0 = 2.0 * l - q;
    return vec3<f32>(
        hsl_hue_to_rgb(p0, q, h_norm + 1.0 / 3.0),
        hsl_hue_to_rgb(p0, q, h_norm),
        hsl_hue_to_rgb(p0, q, h_norm - 1.0 / 3.0)
    );
}

fn hsl_apply(c_in: vec3<f32>) -> vec3<f32> {
    let cc = clamp(c_in, vec3<f32>(0.0), vec3<f32>(HSL_INPUT_CEILING));
    let hsl = hsl_rgb_to_hsl(cc);
    if (hsl.y < HSL_MIN_SAT) { return c_in; }
    var centers: array<f32, HSL_BANDS> = HSL_BAND_CENTERS_DEG;
    let sigma2 = HSL_BAND_SIGMA_DEG * HSL_BAND_SIGMA_DEG;
    var w: array<f32, HSL_BANDS>;
    var w_sum: f32 = 0.0;
    for (var i: i32 = 0; i < HSL_BANDS; i = i + 1) {
        let d = op_hue_dist(hsl.x, centers[i]);
        w[i] = exp(-(d * d) / (2.0 * sigma2));
        w_sum = w_sum + w[i];
    }
    if (w_sum > 1.0) {
        for (var i: i32 = 0; i < HSL_BANDS; i = i + 1) {
            w[i] = w[i] / w_sum;
        }
    }
    let gate = smoothstep(HSL_SAT_GATE_LO, HSL_SAT_GATE_HI, hsl.y);
    var hue_d: f32 = 0.0;
    var sat_d: f32 = 0.0;
    var lum_d: f32 = 0.0;
    for (var i: i32 = 0; i < HSL_BANDS; i = i + 1) {
        hue_d = hue_d + (p.hsl[i].x / HSL_PARAM_FULL_SCALE * HSL_HUE_SHIFT_DEG) * w[i];
        sat_d = sat_d + (p.hsl[i].y / HSL_PARAM_FULL_SCALE) * w[i];
        lum_d = lum_d + (p.hsl[i].z / HSL_PARAM_FULL_SCALE) * w[i];
    }
    hue_d = hue_d * gate;
    sat_d = sat_d * gate;
    lum_d = lum_d * gate;
    let new_h = hsl.x + hue_d;
    let new_s = clamp(hsl.y * (1.0 + sat_d), 0.0, 1.0);
    let new_l = clamp(hsl.z + lum_d * HSL_LUM_SHIFT_SCALE, 0.0, 1.0);
    return hsl_hsl_to_rgb(vec3<f32>(new_h, new_s, new_l));
}
