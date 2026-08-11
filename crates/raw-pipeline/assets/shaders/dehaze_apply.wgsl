struct Params {
    size: vec2<u32>,
    size_lo: vec2<u32>,
    atm: vec4<f32>,
    amount: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var rgb_in: texture_2d<f32>;
@group(0) @binding(2) var ab_mean: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

fn sample_ab(uv: vec2<f32>) -> vec2<f32> {
    let sp = uv * vec2<f32>(p.size_lo) - vec2<f32>(0.5);
    let base = floor(sp);
    let f = sp - base;
    let hi = vec2<i32>(p.size_lo) - vec2<i32>(1);
    let x0 = clamp(i32(base.x), 0, hi.x);
    let y0 = clamp(i32(base.y), 0, hi.y);
    let x1 = clamp(i32(base.x) + 1, 0, hi.x);
    let y1 = clamp(i32(base.y) + 1, 0, hi.y);
    let c00 = textureLoad(ab_mean, vec2<i32>(x0, y0), 0).rg;
    let c10 = textureLoad(ab_mean, vec2<i32>(x1, y0), 0).rg;
    let c01 = textureLoad(ab_mean, vec2<i32>(x0, y1), 0).rg;
    let c11 = textureLoad(ab_mean, vec2<i32>(x1, y1), 0).rg;
    return mix(mix(c00, c10, f.x), mix(c01, c11, f.x), f.y);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let c = textureLoad(rgb_in, pos, 0).rgb;
    let uv = (vec2<f32>(pos) + vec2<f32>(0.5)) / vec2<f32>(p.size);
    let ab = sample_ab(uv);
    let cc = clamp(c, vec3<f32>(0.0), vec3<f32>(1.0));
    let g = 0.2126 * cc.r + 0.7152 * cc.g + 0.0722 * cc.b;
    let t = clamp(ab.x * g + ab.y, 0.0, 1.0);
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
