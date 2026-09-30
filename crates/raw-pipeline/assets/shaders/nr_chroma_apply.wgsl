struct NrApplyParams {
    curves: vec4<f32>,
    size: vec2<u32>,
    lo_size: vec2<u32>,
    lambda: f32,
    mu: f32,
    keep: f32,
}

@group(0) @binding(0) var<uniform> p: NrApplyParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var base: texture_2d<f32>;
@group(0) @binding(3) var den: texture_2d<f32>;
@group(0) @binding(4) var dst: texture_storage_2d<rgba16float, write>;

fn fine_at(q: vec2<u32>) -> vec2<f32> {
    let chroma = to_chroma(textureLoad(src, vec2<i32>(q), 0).rgb).yz;
    return chroma - upsample(base, q, p.lo_size).yz;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let last = vec2<i32>(p.size) - 1;
    var total = vec2<f32>(0.0);
    for (var k = 0; k < 9; k += 1) {
        let q = clamp(vec2<i32>(gid.xy) + vec2<i32>(k % 3 - 1, k / 3 - 1), vec2<i32>(0), last);
        let f = fine_at(vec2<u32>(q));
        total += f * f;
    }
    let f0 = fine_at(gid.xy);
    let centre = f0 * f0;
    let coarse = upsample(den, gid.xy, p.lo_size);
    var chroma = coarse.yz + f0;
    for (var ch = 0u; ch < 2u; ch += 1u) {
        let variance = noise_variance(p.curves[2u * ch], p.curves[2u * ch + 1u], coarse.a);
        let g = shrink_gain(local_energy(total[ch], centre[ch]), variance, p.lambda, p.mu, p.keep);
        chroma[ch] += (g - 1.0) * f0[ch];
    }
    let px = textureLoad(src, vec2<i32>(gid.xy), 0);
    textureStore(dst, vec2<i32>(gid.xy), vec4<f32>(from_chroma(nr_luma(px.rgb), chroma.x, chroma.y), px.a));
}
