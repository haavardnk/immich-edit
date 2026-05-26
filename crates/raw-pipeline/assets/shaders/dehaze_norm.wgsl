struct Params {
    size: vec2<u32>,
    _pad: vec2<u32>,
    atm: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(src, pos, 0).rgb;
    let r = clamp(c.r, 0.0, 1.0) / max(p.atm.r, 1e-5);
    let g = clamp(c.g, 0.0, 1.0) / max(p.atm.g, 1e-5);
    let b = clamp(c.b, 0.0, 1.0) / max(p.atm.b, 1e-5);
    let m = clamp(min(r, min(g, b)), 0.0, 1.0);
    textureStore(dst, pos, vec4<f32>(m, 0.0, 0.0, 1.0));
}
