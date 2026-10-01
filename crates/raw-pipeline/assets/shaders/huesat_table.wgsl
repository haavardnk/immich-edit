fn huesat_srgb_gamma(c: f32) -> f32 {
    if (c <= 0.0031308) {
        return 12.92 * c;
    }
    return 1.055 * pow(max(c, 0.0), 1.0 / 2.4) - 0.055;
}

fn huesat_srgb_degamma(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn huesat_rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let cmax = max(rgb.r, max(rgb.g, rgb.b));
    let cmin = min(rgb.r, min(rgb.g, rgb.b));
    let d = cmax - cmin;
    var h = 0.0;
    if (d > 0.0) {
        if (cmax == rgb.r) {
            h = ((rgb.g - rgb.b) / d) % 6.0;
        } else if (cmax == rgb.g) {
            h = (rgb.b - rgb.r) / d + 2.0;
        } else {
            h = (rgb.r - rgb.g) / d + 4.0;
        }
    }
    if (h < 0.0) {
        h = h + 6.0;
    }
    var s = 0.0;
    if (cmax > 0.0) {
        s = d / cmax;
    }
    return vec3<f32>(h, s, cmax);
}

fn huesat_hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    var h = hsv.x % 6.0;
    if (h < 0.0) {
        h = h + 6.0;
    }
    let s = clamp(hsv.y, 0.0, 1.0);
    let v = hsv.z;
    if (s <= 0.0) {
        return vec3<f32>(v, v, v);
    }
    let i = floor(h);
    let f = h - i;
    let pp = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let idx = i32(i);
    if (idx == 0) { return vec3<f32>(v, t, pp); }
    if (idx == 1) { return vec3<f32>(q, v, pp); }
    if (idx == 2) { return vec3<f32>(pp, v, t); }
    if (idx == 3) { return vec3<f32>(pp, q, v); }
    if (idx == 4) { return vec3<f32>(t, pp, v); }
    return vec3<f32>(v, pp, q);
}

fn huesat_any_in_range(c: vec3<f32>) -> bool {
    return (c.r >= 0.0 && c.r <= 1.0)
        || (c.g >= 0.0 && c.g <= 1.0)
        || (c.b >= 0.0 && c.b <= 1.0);
}

fn huesat_sample(table: texture_3d<f32>, dims: vec3<u32>, hsv: vec3<f32>) -> vec3<f32> {
    let hue_div = i32(dims.x);
    let sat_div = i32(dims.y);
    let val_div = max(i32(dims.z), 1);

    let h_scaled = hsv.x / 6.0 * f32(hue_div);
    let s_scaled = hsv.y * f32(max(sat_div - 1, 1));
    var v_scaled = 0.0;
    if (val_div > 1) {
        v_scaled = clamp(hsv.z, 0.0, 1.0) * f32(val_div - 1);
    }

    let h0f = floor(h_scaled);
    let hf = h_scaled - h0f;
    let h0 = ((i32(h0f) % hue_div) + hue_div) % hue_div;
    let h1 = (h0 + 1) % hue_div;

    let s0f = min(floor(s_scaled), f32(sat_div - 1));
    let sf = clamp(s_scaled - s0f, 0.0, 1.0);
    let s0 = i32(s0f);
    let s1 = min(s0 + 1, sat_div - 1);

    let v0f = min(floor(v_scaled), f32(max(val_div - 1, 0)));
    let vf = clamp(v_scaled - v0f, 0.0, 1.0);
    let v0 = i32(v0f);
    let v1 = min(v0 + 1, val_div - 1);

    let c000 = textureLoad(table, vec3<i32>(h0, s0, v0), 0).rgb;
    let c100 = textureLoad(table, vec3<i32>(h1, s0, v0), 0).rgb;
    let c010 = textureLoad(table, vec3<i32>(h0, s1, v0), 0).rgb;
    let c110 = textureLoad(table, vec3<i32>(h1, s1, v0), 0).rgb;
    let c00 = mix(c000, c100, hf);
    let c10 = mix(c010, c110, hf);
    let cv0 = mix(c00, c10, sf);
    if (val_div <= 1) {
        return cv0;
    }
    let c001 = textureLoad(table, vec3<i32>(h0, s0, v1), 0).rgb;
    let c101 = textureLoad(table, vec3<i32>(h1, s0, v1), 0).rgb;
    let c011 = textureLoad(table, vec3<i32>(h0, s1, v1), 0).rgb;
    let c111 = textureLoad(table, vec3<i32>(h1, s1, v1), 0).rgb;
    let c01 = mix(c001, c101, hf);
    let c11 = mix(c011, c111, hf);
    let cv1 = mix(c01, c11, sf);
    return mix(cv0, cv1, vf);
}

fn huesat_map(table: texture_3d<f32>, dims: vec4<u32>, rgb: vec3<f32>, bounded: bool) -> vec3<f32> {
    if (bounded && !huesat_any_in_range(rgb)) {
        return rgb;
    }
    if (!bounded && any(rgb < vec3<f32>(0.0))) {
        return rgb;
    }
    var source = rgb;
    if (bounded) {
        source = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    }
    let srgb_enc = dims.w == 1u;
    var hsv = huesat_rgb_to_hsv(source);
    var encoded_v = hsv.z;
    if (srgb_enc) {
        encoded_v = huesat_srgb_gamma(clamp(hsv.z, 0.0, 1.0));
    }
    let delta = huesat_sample(table, dims.xyz, vec3<f32>(hsv.x, hsv.y, encoded_v));
    hsv.x = hsv.x + delta.x / 60.0;
    hsv.x = hsv.x - 6.0 * floor(hsv.x / 6.0);
    hsv.y = hsv.y * delta.y;
    if (srgb_enc) {
        hsv.z = huesat_srgb_degamma(clamp(encoded_v * delta.z, 0.0, 1.0));
    } else {
        hsv.z = hsv.z * delta.z;
    }
    if (bounded) {
        hsv.y = clamp(hsv.y, 0.0, 1.0);
        hsv.z = clamp(hsv.z, 0.0, 1.0);
    }
    return huesat_hsv_to_rgb(hsv);
}
