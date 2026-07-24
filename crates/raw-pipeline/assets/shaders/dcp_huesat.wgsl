struct Params {
    dims: vec4<u32>,
    to_pp0: vec4<f32>,
    to_pp1: vec4<f32>,
    to_pp2: vec4<f32>,
    from_pp0: vec4<f32>,
    from_pp1: vec4<f32>,
    from_pp2: vec4<f32>,
    flags: vec4<u32>,
    tone_lut: array<vec4<f32>, 64>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var lut_tex: texture_3d<f32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba16float, write>;

fn to_pp(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(p.to_pp0.xyz, c),
        dot(p.to_pp1.xyz, c),
        dot(p.to_pp2.xyz, c),
    );
}

fn from_pp(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(p.from_pp0.xyz, c),
        dot(p.from_pp1.xyz, c),
        dot(p.from_pp2.xyz, c),
    );
}

fn srgb_gamma(c: f32) -> f32 {
    if (c <= 0.0031308) {
        return 12.92 * c;
    }
    return 1.055 * pow(max(c, 0.0), 1.0 / 2.4) - 0.055;
}

fn srgb_degamma(c: f32) -> f32 {
    if (c <= 0.04045) {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
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

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
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

fn table_at(h: i32, s: i32, v: i32) -> vec3<f32> {
    return textureLoad(lut_tex, vec3<i32>(h, s, v), 0).rgb;
}

fn sample_huesat(hsv: vec3<f32>) -> vec3<f32> {
    let hue_div = i32(p.dims.x);
    let sat_div = i32(p.dims.y);
    let val_div = max(i32(p.dims.z), 1);

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

    let c000 = table_at(h0, s0, v0);
    let c100 = table_at(h1, s0, v0);
    let c010 = table_at(h0, s1, v0);
    let c110 = table_at(h1, s1, v0);
    let c00 = mix(c000, c100, hf);
    let c10 = mix(c010, c110, hf);
    let cv0 = mix(c00, c10, sf);
    if (val_div <= 1) {
        return cv0;
    }
    let c001 = table_at(h0, s0, v1);
    let c101 = table_at(h1, s0, v1);
    let c011 = table_at(h0, s1, v1);
    let c111 = table_at(h1, s1, v1);
    let c01 = mix(c001, c101, hf);
    let c11 = mix(c011, c111, hf);
    let cv1 = mix(c01, c11, sf);
    return mix(cv0, cv1, vf);
}

fn tone_lut_get(k: i32) -> f32 {
    let v = p.tone_lut[k / 4];
    let m = k % 4;
    if (m == 0) { return v.x; }
    if (m == 1) { return v.y; }
    if (m == 2) { return v.z; }
    return v.w;
}

fn tone_lut_sample(x: f32) -> f32 {
    let pos = clamp(x, 0.0, 1.0) * 255.0;
    let i0 = i32(floor(pos));
    let i1 = min(i0 + 1, 255);
    let f = pos - f32(i0);
    return mix(tone_lut_get(i0), tone_lut_get(i1), f);
}

fn tone_ordered(hi: f32, mid: f32, lo: f32) -> vec3<f32> {
    let hi_out = tone_lut_sample(hi);
    let lo_out = tone_lut_sample(lo);
    if (hi - lo <= 1e-8) {
        return vec3<f32>(lo_out);
    }
    let mid_out = lo_out + (hi_out - lo_out) * (mid - lo) / (hi - lo);
    return vec3<f32>(hi_out, mid_out, lo_out);
}

fn adobe_tone(c: vec3<f32>) -> vec3<f32> {
    let r = clamp(c.r, 0.0, 1.0);
    let g = clamp(c.g, 0.0, 1.0);
    let b = clamp(c.b, 0.0, 1.0);
    if (r >= g) {
        if (g > b) {
            let t = tone_ordered(r, g, b);
            return t;
        }
        if (b > r) {
            let t = tone_ordered(b, r, g);
            return vec3<f32>(t.y, t.z, t.x);
        }
        if (b > g) {
            let t = tone_ordered(r, b, g);
            return vec3<f32>(t.x, t.z, t.y);
        }
        let v = tone_lut_sample(r);
        return vec3<f32>(v);
    }
    if (r >= b) {
        let t = tone_ordered(g, r, b);
        return vec3<f32>(t.y, t.x, t.z);
    }
    if (b > g) {
        let t = tone_ordered(b, g, r);
        return vec3<f32>(t.z, t.y, t.x);
    }
    let t = tone_ordered(g, b, r);
    return vec3<f32>(t.z, t.x, t.y);
}

fn luma(c: vec3<f32>) -> f32 {
    return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
}

fn project_gamut(c: vec3<f32>) -> vec3<f32> {
    let neutral = clamp(luma(c), 0.0, 1.0);
    var out = c;
    let mn = min(out.r, min(out.g, out.b));
    if (mn < 0.0) {
        let t = clamp(-mn / (neutral - mn), 0.0, 1.0);
        out = mix(out, vec3<f32>(neutral), t);
    }
    let mx = max(out.r, max(out.g, out.b));
    if (mx > 1.0) {
        let t = clamp((mx - 1.0) / (mx - neutral), 0.0, 1.0);
        out = mix(out, vec3<f32>(neutral), t);
    }
    return out;
}

fn any_channel_in_range(c: vec3<f32>) -> bool {
    return (c.r >= 0.0 && c.r <= 1.0)
        || (c.g >= 0.0 && c.g <= 1.0)
        || (c.b >= 0.0 && c.b <= 1.0);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(out_tex);
    if (gid.x >= size.x || gid.y >= size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let src = textureLoad(src_tex, coord, 0);
    let output = p.flags.x == 1u;
    let apply_table = p.flags.y == 1u;
    let apply_tone = p.flags.z == 1u;
    let pp_source = to_pp(src.rgb);
    var pp = pp_source;
    let srgb_enc = p.dims.w == 1u;
    if (apply_table) {
        let in_range = any_channel_in_range(pp_source);
        let can_apply = select(all(pp_source >= vec3<f32>(0.0)), in_range, output);
        if (can_apply) {
            let table_rgb = select(pp_source, clamp(pp_source, vec3<f32>(0.0), vec3<f32>(1.0)), output);
            var hsv = rgb_to_hsv(table_rgb);
            var encoded_v = hsv.z;
            if (srgb_enc) {
                encoded_v = srgb_gamma(clamp(hsv.z, 0.0, 1.0));
            }
            let delta = sample_huesat(vec3<f32>(hsv.x, hsv.y, encoded_v));
            hsv.x = hsv.x + delta.x / 60.0;
            hsv.x = hsv.x - 6.0 * floor(hsv.x / 6.0);
            hsv.y = hsv.y * delta.y;
            if (srgb_enc) {
                hsv.z = srgb_degamma(clamp(encoded_v * delta.z, 0.0, 1.0));
            } else {
                hsv.z = hsv.z * delta.z;
            }
            if (output) {
                hsv.y = clamp(hsv.y, 0.0, 1.0);
                hsv.z = clamp(hsv.z, 0.0, 1.0);
            }
            pp = hsv_to_rgb(hsv);
        }
    }
    if (apply_tone && any_channel_in_range(pp)) {
        pp = adobe_tone(pp);
    }
    let lin = from_pp(pp);
    if (output) {
        let mapped = project_gamut(lin);
        let display = vec3<f32>(
            srgb_gamma(clamp(mapped.r, 0.0, 1.0)),
            srgb_gamma(clamp(mapped.g, 0.0, 1.0)),
            srgb_gamma(clamp(mapped.b, 0.0, 1.0)),
        );
        textureStore(out_tex, coord, vec4<f32>(display, src.a));
    } else {
        textureStore(out_tex, coord, vec4<f32>(lin, src.a));
    }
}
