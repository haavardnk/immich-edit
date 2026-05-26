struct Params {
    size: vec2<u32>,
    radius: u32,
    axis: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let r = i32(p.radius);
    var s = vec4<f32>(0.0);
    var n: f32 = 0.0;
    let x0 = i32(gid.x);
    let y0 = i32(gid.y);
    let wmax = i32(p.size.x);
    let hmax = i32(p.size.y);
    for (var i: i32 = -r; i <= r; i = i + 1) {
        var xi = x0;
        var yi = y0;
        var ok = true;
        if (p.axis == 0u) {
            xi = x0 + i;
            if (xi < 0 || xi >= wmax) { ok = false; }
        } else {
            yi = y0 + i;
            if (yi < 0 || yi >= hmax) { ok = false; }
        }
        if (ok) {
            s = s + textureLoad(src, vec2<i32>(xi, yi), 0);
            n = n + 1.0;
        }
    }
    textureStore(dst, vec2<i32>(x0, y0), s / max(n, 1.0));
}
