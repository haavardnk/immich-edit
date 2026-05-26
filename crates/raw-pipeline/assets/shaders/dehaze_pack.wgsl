struct Params {
    size: vec2<u32>,
    _pad: vec2<u32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var rgb_in: texture_2d<f32>;
@group(0) @binding(2) var dn_min: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(rgb_in, pos, 0).rgb;
    let dn = textureLoad(dn_min, pos, 0).r;
    let cc = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let lum = 0.2126 * cc.r + 0.7152 * cc.g + 0.0722 * cc.b;
    let pr = clamp(1.0 - 0.95 * dn, 0.0, 1.0);
    textureStore(dst, pos, vec4<f32>(lum, pr, lum * lum, lum * pr));
}
