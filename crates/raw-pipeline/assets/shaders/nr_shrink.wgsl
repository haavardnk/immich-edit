struct NrShrinkParams {
    curves: vec4<f32>,
    size: vec2<u32>,
    step: u32,
    lo: u32,
    count: u32,
    init: u32,
    lambda: f32,
    mu: f32,
    keep: f32,
}

@group(0) @binding(0) var<uniform> p: NrShrinkParams;
@group(0) @binding(1) var cur: texture_2d<f32>;
@group(0) @binding(2) var next: texture_2d<f32>;
@group(0) @binding(3) var acc_in: texture_2d<f32>;
@group(0) @binding(4) var dst: texture_storage_2d<rgba16float, write>;

fn detail_at(q: vec2<i32>) -> vec4<f32> {
    return textureLoad(cur, q, 0) - textureLoad(next, q, 0);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let c = vec2<i32>(gid.xy);
    let last = vec2<i32>(p.size) - 1;
    let s = i32(p.step);
    var total = vec4<f32>(0.0);
    for (var k = 0; k < 9; k += 1) {
        let d = detail_at(clamp(c + vec2<i32>(k % 3 - 1, k / 3 - 1) * s, vec2<i32>(0), last));
        total += d * d;
    }
    let d0 = detail_at(c);
    let centre = d0 * d0;
    let reference = textureLoad(next, c, 0).r;
    var acc = vec4<f32>(0.0);
    if (p.init != INIT_ZERO) {
        acc = textureLoad(acc_in, c, 0);
    }
    if (p.init == INIT_CARRY_REFERENCE) {
        acc.a = reference;
    }
    for (var ch = 0u; ch < p.count; ch += 1u) {
        let k = p.lo + ch;
        let variance = noise_variance(p.curves[2u * ch], p.curves[2u * ch + 1u], reference);
        let g = shrink_gain(local_energy(total[k], centre[k]), variance, p.lambda, p.mu, p.keep);
        acc[k] += (g - 1.0) * d0[k];
    }
    textureStore(dst, c, acc);
}
