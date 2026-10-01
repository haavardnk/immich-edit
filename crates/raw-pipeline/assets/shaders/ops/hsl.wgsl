fn hsl_hue_sat(c_in: vec3<f32>) -> vec2<f32> {
    let k = max(max(max(c_in.r, c_in.g), c_in.b), 1.0);
    let c = c_in / k;
    let mx = max(max(c.r, c.g), c.b);
    let mn = min(min(c.r, c.g), c.b);
    let d = mx - mn;
    if (d < 1e-6) { return vec2<f32>(0.0); }
    var s: f32;
    if (mx + mn > 1.0) { s = d / (2.0 - mx - mn); } else { s = d / (mx + mn); }
    var h: f32;
    if (mx == c.r) {
        var k6 = (c.g - c.b) / d;
        if (c.g < c.b) { k6 = k6 + 6.0; }
        h = k6;
    } else if (mx == c.g) {
        h = (c.b - c.r) / d + 2.0;
    } else {
        h = (c.r - c.g) / d + 4.0;
    }
    return vec2<f32>(h * 60.0, s);
}

fn hsl_cbrt(x: vec3<f32>) -> vec3<f32> {
    return sign(x) * pow(abs(x), vec3<f32>(1.0 / 3.0));
}

fn hsl_to_oklab(c: vec3<f32>) -> vec3<f32> {
    let lms = hsl_cbrt(vec3<f32>(
        0.41222146 * c.r + 0.53633255 * c.g + 0.051445995 * c.b,
        0.2119035 * c.r + 0.6806995 * c.g + 0.10739696 * c.b,
        0.08830246 * c.r + 0.28171885 * c.g + 0.6299787 * c.b,
    ));
    return vec3<f32>(
        0.21045426 * lms.x + 0.7936178 * lms.y - 0.004072047 * lms.z,
        1.9779985 * lms.x - 2.4285922 * lms.y + 0.4505937 * lms.z,
        0.025904037 * lms.x + 0.78277177 * lms.y - 0.80867577 * lms.z,
    );
}

fn hsl_from_oklab(lab: vec3<f32>) -> vec3<f32> {
    let l_ = lab.x + 0.39633778 * lab.y + 0.21580376 * lab.z;
    let m_ = lab.x - 0.105561346 * lab.y - 0.06385417 * lab.z;
    let s_ = lab.x - 0.08948418 * lab.y - 1.2914855 * lab.z;
    let l3 = l_ * l_ * l_;
    let m3 = m_ * m_ * m_;
    let s3 = s_ * s_ * s_;
    return vec3<f32>(
        4.0767417 * l3 - 3.3077116 * m3 + 0.23096994 * s3,
        -1.268438 * l3 + 2.6097574 * m3 - 0.34131938 * s3,
        -0.0041960863 * l3 - 0.7034186 * m3 + 1.7076147 * s3,
    );
}

fn hsl_apply(c_in: vec3<f32>) -> vec3<f32> {
    let hs = hsl_hue_sat(max(c_in, vec3<f32>(0.0)));
    if (hs.y < HSL_MIN_SAT) { return c_in; }
    var centers: array<f32, HSL_BANDS> = HSL_BAND_CENTERS_DEG;
    let sigma2 = HSL_BAND_SIGMA_DEG * HSL_BAND_SIGMA_DEG;
    var w: array<f32, HSL_BANDS>;
    var w_sum: f32 = 0.0;
    for (var i: i32 = 0; i < HSL_BANDS; i = i + 1) {
        let d = op_hue_dist(hs.x, centers[i]);
        w[i] = exp(-(d * d) / (2.0 * sigma2));
        w_sum = w_sum + w[i];
    }
    if (w_sum > 1.0) {
        for (var i: i32 = 0; i < HSL_BANDS; i = i + 1) {
            w[i] = w[i] / w_sum;
        }
    }
    let gate = smoothstep(HSL_SAT_GATE_LO, HSL_SAT_GATE_HI, hs.y);
    var hue_d: f32 = 0.0;
    var sat_d: f32 = 0.0;
    var lum_d: f32 = 0.0;
    for (var i: i32 = 0; i < HSL_BANDS; i = i + 1) {
        hue_d = hue_d + (p.hsl[i].x / HSL_PARAM_FULL_SCALE * HSL_HUE_SHIFT_DEG) * w[i];
        sat_d = sat_d + (p.hsl[i].y / HSL_PARAM_FULL_SCALE) * w[i];
        lum_d = lum_d + (p.hsl[i].z / HSL_PARAM_FULL_SCALE) * w[i];
    }
    let angle = radians(hue_d * gate);
    let sn = sin(angle);
    let cs = cos(angle);
    let chroma = max(1.0 + sat_d * gate, 0.0);
    let lab = hsl_to_oklab(c_in);
    let rotated = vec3<f32>(
        lab.x,
        (lab.y * cs - lab.z * sn) * chroma,
        (lab.y * sn + lab.z * cs) * chroma,
    );
    return hsl_from_oklab(rotated) * exp2(lum_d * gate * HSL_LUM_EV);
}
