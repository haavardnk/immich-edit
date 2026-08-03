// color-space: linear scene-referred Rgba16Float in/out; separable gaussian over the stroke patch
struct Params {
    img: vec2<u32>,
    bbox_origin: vec2<u32>,
    bbox_size: vec2<u32>,
    n_points: u32,
    mode: u32,
    offset: vec2<f32>,
    radius: f32,
    hardness: f32,
    opacity: f32,
    sigma: f32,
    dir: u32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var in_tex: texture_2d<f32>;
@group(0) @binding(2) var out_tex: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.bbox_size.x || gid.y >= p.bbox_size.y) { return; }
    let s = max(p.sigma, 0.01);
    let r = i32(ceil(s * 3.0));
    let two_s2 = 2.0 * s * s;
    let max_x = i32(p.bbox_size.x) - 1;
    let max_y = i32(p.bbox_size.y) - 1;
    var acc = vec3<f32>(0.0);
    var wsum = 0.0;
    for (var k = -r; k <= r; k = k + 1) {
        let fk = f32(k);
        let wt = exp(-(fk * fk) / two_s2);
        var sx = i32(gid.x);
        var sy = i32(gid.y);
        if (p.dir == 0u) {
            sx = clamp(sx + k, 0, max_x);
        } else {
            sy = clamp(sy + k, 0, max_y);
        }
        acc = acc + wt * textureLoad(in_tex, vec2<i32>(sx, sy), 0).rgb;
        wsum = wsum + wt;
    }
    textureStore(out_tex, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(acc / wsum, 1.0));
}
