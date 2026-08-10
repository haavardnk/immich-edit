struct Params {
    size: vec2<u32>,
    radius: i32,
    axis: i32,
    mode: u32,
    _pad: u32,
    k: array<vec4<f32>, 4>,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var aux: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<r32float, write>;

const EPS: f32 = 1e-5;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let last_x = i32(p.size.x) - 1;
    let last_y = i32(p.size.y) - 1;
    var acc = 0.0;
    for (var i = 0; i <= 2 * p.radius; i = i + 1) {
        var sx = x;
        var sy = y;
        if (p.axis == 0) {
            sx = clamp(x + i - p.radius, 0, last_x);
        } else {
            sy = clamp(y + i - p.radius, 0, last_y);
        }
        acc = acc + p.k[i / 4][i % 4] * textureLoad(src, vec2<i32>(sx, sy), 0).r;
    }
    let c = vec2<i32>(x, y);
    var out = acc;
    if (p.mode == 1u) {
        out = textureLoad(aux, c, 0).r / max(acc, EPS);
    } else if (p.mode == 2u) {
        out = textureLoad(aux, c, 0).r * acc;
    }
    textureStore(dst, c, vec4<f32>(out, 0.0, 0.0, 0.0));
}
