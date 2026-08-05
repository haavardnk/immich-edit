struct LutParams {
    dims: vec4<u32>,
    domain_min: vec4<f32>,
    domain_max: vec4<f32>,
    misc: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: LutParams;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var lut_tex: texture_3d<f32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;

// TONE_WGSL_INJECT

fn lut_at(coord: vec3<i32>) -> vec3<f32> {
    return textureLoad(lut_tex, coord, 0).rgb;
}

fn lut_sample(rgb: vec3<f32>) -> vec3<f32> {
    let n = i32(p.dims.z);
    let last = f32(n - 1);
    let span = p.domain_max.xyz - p.domain_min.xyz;
    let normalized = clamp((rgb - p.domain_min.xyz) / span, vec3<f32>(0.0), vec3<f32>(1.0));
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
    let width = p.dims.x;
    let height = p.dims.y;
    if (gid.x >= width || gid.y >= height) { return; }
    let coord = vec2<i32>(i32(gid.x), i32(gid.y));
    let src = textureLoad(src_tex, coord, 0);
    let amount = p.misc.x;
    let sampled = lut_sample(src.rgb);
    let blended = clamp(src.rgb + amount * (sampled - src.rgb), vec3<f32>(0.0), vec3<f32>(1.0));
    var alpha = src.a;
    if (alpha != 0.0) {
        alpha = warn_clip_alpha(blended);
    }
    textureStore(out_tex, coord, vec4<f32>(blended, alpha));
}
