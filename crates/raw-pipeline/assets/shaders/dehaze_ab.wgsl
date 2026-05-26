struct Params {
    size: vec2<u32>,
    _pad: vec2<u32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var means: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let m = textureLoad(means, pos, 0);
    let mi = m.r;
    let mp = m.g;
    let mii = m.b;
    let mip = m.a;
    let var_i = max(mii - mi * mi, 0.0);
    let cov_ip = mip - mi * mp;
    let a = cov_ip / (var_i + 1e-3);
    let b = mp - a * mi;
    textureStore(dst, pos, vec4<f32>(a, b, 0.0, 1.0));
}
