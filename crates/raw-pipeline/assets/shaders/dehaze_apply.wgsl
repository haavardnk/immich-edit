struct Params {
    size: vec2<u32>,
    _pad0: vec2<u32>,
    atm: vec4<f32>,
    amount: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var rgb_in: texture_2d<f32>;
@group(0) @binding(2) var ab_mean: texture_2d<f32>;
@group(0) @binding(3) var samp: sampler;
@group(0) @binding(4) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(rgb_in, pos, 0).rgb;
    let uv = (vec2<f32>(pos) + vec2<f32>(0.5)) / vec2<f32>(p.size);
    let ab = textureSampleLevel(ab_mean, samp, uv, 0.0);
    let cc = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let g = 0.2126 * cc.r + 0.7152 * cc.g + 0.0722 * cc.b;
    let t = clamp(ab.r * g + ab.g, 0.0, 1.0);
    let atm = p.atm.rgb;
    let amt = p.amount.x;
    var outc: vec3<f32>;
    if (amt > 0.0) {
        let ti = max(t, 0.16);
        let jr = (c.r - atm.r) / ti + atm.r;
        let jg = (c.g - atm.g) / ti + atm.g;
        let jb = (c.b - atm.b) / ti + atm.b;
        let j = max(vec3<f32>(jr, jg, jb), vec3<f32>(0.0));
        outc = c + (j - c) * amt;
    } else {
        let neg = -amt;
        let t_add = clamp(1.0 - t * neg * 0.5, 0.0, 1.0);
        outc = atm * (1.0 - t_add) + c * t_add;
    }
    textureStore(dst, pos, vec4<f32>(outc, 1.0));
}
