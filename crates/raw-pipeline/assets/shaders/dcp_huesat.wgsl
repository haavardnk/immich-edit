struct Params {
    dims: vec4<u32>,
    to_pp: array<vec4<f32>, 3>,
    from_pp: array<vec4<f32>, 3>,
    flags: vec4<u32>,
    tone_lut: array<vec4<f32>, 64>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var lut_tex: texture_3d<f32>;
// DISPLAY_STORE_INJECT

// TONE_WGSL_INJECT

// DISPLAY_CURVES_INJECT

// HUESAT_TABLE_INJECT

fn to_pp(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(p.to_pp[0].xyz, c),
        dot(p.to_pp[1].xyz, c),
        dot(p.to_pp[2].xyz, c),
    );
}

fn from_pp(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        dot(p.from_pp[0].xyz, c),
        dot(p.from_pp[1].xyz, c),
        dot(p.from_pp[2].xyz, c),
    );
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

fn profile_tone(c: vec3<f32>) -> vec3<f32> {
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

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(out_tex);
    if (gid.x >= size.x || gid.y >= size.y) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let src = textureLoad(src_tex, coord, 0);
    let apply_table = p.flags.x == 1u;
    let apply_tone = p.flags.y == 1u;
    let warn = p.flags.z;
    var pp = to_pp(src.rgb);
    if (apply_table) {
        pp = huesat_map(lut_tex, p.dims, pp, true);
    }
    if (apply_tone && huesat_any_in_range(pp)) {
        pp = profile_tone(pp);
    }
    let lin = from_pp(pp);
    let p3 = (warn >> 2u) & 1u;
    let display = display_curves_apply(tone_rgb_cs(lin, p3));
    var alpha = src.a;
    if ((warn & 1u) != 0u) {
        alpha = select(1.0, 0.0, tone_is_out_of_gamut(lin, p3));
    }
    if (alpha != 0.0 && (warn & 2u) != 0u) {
        alpha = warn_clip_alpha(display);
    }
    store_display(coord, vec4<f32>(display, alpha));
}
