struct LutParams {
    size: vec2<u32>,
    cube_size: u32,
    shaper_size: u32,
    cube_min: vec3<f32>,
    shaper_width: u32,
    cube_max: vec3<f32>,
    amount: f32,
    shaper_min: vec3<f32>,
    display_p3: u32,
    shaper_max: vec3<f32>,
};

@group(0) @binding(0) var<uniform> p: LutParams;
// DISPLAY_LOAD_INJECT
@group(0) @binding(2) var lut_tex: texture_3d<f32>;
// DISPLAY_STORE_INJECT
@group(0) @binding(4) var shaper_tex: texture_2d<f32>;

// TONE_WGSL_INJECT

fn srgb_decode(c: vec3<f32>) -> vec3<f32> {
    let v = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return select(pow((v + 0.055) / 1.055, vec3<f32>(2.4)), v / 12.92, v <= vec3<f32>(0.04045));
}

fn srgb_encode(c: vec3<f32>) -> vec3<f32> {
    let v = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec3<f32>(tone_srgb_oetf(v.x), tone_srgb_oetf(v.y), tone_srgb_oetf(v.z));
}

fn to_lut_space(display: vec3<f32>) -> vec3<f32> {
    if (p.display_p3 == 0u) { return display; }
    let linear = tone_from_output_space(srgb_decode(display), 1u);
    return srgb_encode(tone_map_to_gamut(linear, 0u));
}

fn from_lut_space(srgb: vec3<f32>) -> vec3<f32> {
    if (p.display_p3 == 0u) { return srgb; }
    return srgb_encode(tone_to_output_space(srgb_decode(srgb), 1u));
}

fn shaper_at(i: u32) -> vec3<f32> {
    let w = p.shaper_width;
    return textureLoad(shaper_tex, vec2<i32>(i32(i % w), i32(i / w)), 0).rgb;
}

fn shaper_sample(rgb: vec3<f32>) -> vec3<f32> {
    let n = p.shaper_size;
    let span = p.shaper_max - p.shaper_min;
    let pos = clamp((rgb - p.shaper_min) / span, vec3<f32>(0.0), vec3<f32>(1.0)) * f32(n - 1u);
    let i = min(vec3<u32>(pos), vec3<u32>(n - 2u));
    let f = pos - vec3<f32>(i);
    let r = shaper_at(i.x).x;
    let g = shaper_at(i.y).y;
    let b = shaper_at(i.z).z;
    return vec3<f32>(
        r + (shaper_at(i.x + 1u).x - r) * f.x,
        g + (shaper_at(i.y + 1u).y - g) * f.y,
        b + (shaper_at(i.z + 1u).z - b) * f.z,
    );
}

fn lut_at(coord: vec3<i32>) -> vec3<f32> {
    return textureLoad(lut_tex, coord, 0).rgb;
}

fn lut_sample(rgb: vec3<f32>) -> vec3<f32> {
    let n = i32(p.cube_size);
    let last = f32(n - 1);
    let span = p.cube_max - p.cube_min;
    let normalized = clamp((rgb - p.cube_min) / span, vec3<f32>(0.0), vec3<f32>(1.0));
    let coordf = normalized * last;
    let basef = floor(coordf);
    let base = vec3<i32>(
        min(i32(basef.x), n - 2),
        min(i32(basef.y), n - 2),
        min(i32(basef.z), n - 2),
    );
    let hi = base + vec3<i32>(1, 1, 1);
    let fr = coordf.x - f32(base.x);
    let fg = coordf.y - f32(base.y);
    let fb = coordf.z - f32(base.z);
    let c000 = lut_at(base);
    let c111 = lut_at(hi);

    var w: vec3<f32>;
    var v1: vec3<f32>;
    var v2: vec3<f32>;
    if (fr >= fg && fg >= fb) {
        w = vec3<f32>(fr, fg, fb);
        v1 = lut_at(vec3<i32>(hi.x, base.y, base.z));
        v2 = lut_at(vec3<i32>(hi.x, hi.y, base.z));
    } else if (fr >= fb && fb >= fg) {
        w = vec3<f32>(fr, fb, fg);
        v1 = lut_at(vec3<i32>(hi.x, base.y, base.z));
        v2 = lut_at(vec3<i32>(hi.x, base.y, hi.z));
    } else if (fb >= fr && fr >= fg) {
        w = vec3<f32>(fb, fr, fg);
        v1 = lut_at(vec3<i32>(base.x, base.y, hi.z));
        v2 = lut_at(vec3<i32>(hi.x, base.y, hi.z));
    } else if (fg >= fr && fr >= fb) {
        w = vec3<f32>(fg, fr, fb);
        v1 = lut_at(vec3<i32>(base.x, hi.y, base.z));
        v2 = lut_at(vec3<i32>(hi.x, hi.y, base.z));
    } else if (fg >= fb && fb >= fr) {
        w = vec3<f32>(fg, fb, fr);
        v1 = lut_at(vec3<i32>(base.x, hi.y, base.z));
        v2 = lut_at(vec3<i32>(base.x, hi.y, hi.z));
    } else {
        w = vec3<f32>(fb, fg, fr);
        v1 = lut_at(vec3<i32>(base.x, base.y, hi.z));
        v2 = lut_at(vec3<i32>(base.x, hi.y, hi.z));
    }
    return c000 + w.x * (v1 - c000) + w.y * (v2 - v1) + w.z * (c111 - v2);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = p.size.x;
    let height = p.size.y;
    if (gid.x >= width || gid.y >= height) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let src = load_display(coord);
    let input = to_lut_space(src.rgb);
    var graded = input;
    if (p.shaper_size > 0u) { graded = shaper_sample(graded); }
    if (p.cube_size > 0u) { graded = lut_sample(graded); }
    let blended = clamp(input + p.amount * (graded - input), vec3<f32>(0.0), vec3<f32>(1.0));
    let out = from_lut_space(blended);
    var alpha = src.a;
    if (alpha != 0.0) {
        alpha = warn_clip_alpha(out);
    }
    store_display(coord, vec4<f32>(out, alpha));
}
