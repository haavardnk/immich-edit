struct BlurParams {
    sigma: f32,
    radius: f32,
    width: u32,
    height: u32,
    axis: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0) var<uniform> p: BlurParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.width || gid.y >= p.height) { return; }
    let r = i32(ceil(p.radius));
    let two_s2 = 2.0 * p.sigma * p.sigma;
    var acc = vec3<f32>(0.0, 0.0, 0.0);
    var wsum = 0.0;
    let max_x = i32(p.width) - 1;
    let max_y = i32(p.height) - 1;
    for (var k: i32 = -r; k <= r; k = k + 1) {
        let w = exp(-f32(k * k) / two_s2);
        var sx = i32(gid.x);
        var sy = i32(gid.y);
        if (p.axis == 0u) {
            sx = clamp(sx + k, 0, max_x);
        } else {
            sy = clamp(sy + k, 0, max_y);
        }
        let s = textureLoad(src, vec2<i32>(sx, sy), 0).rgb;
        acc = acc + s * w;
        wsum = wsum + w;
    }
    textureStore(dst, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(acc / wsum, 1.0));
}
