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
    var m: f32 = 1.0e30;
    let x0 = i32(gid.x);
    let y0 = i32(gid.y);
    let wmax = i32(p.size.x) - 1;
    let hmax = i32(p.size.y) - 1;
    for (var i: i32 = -r; i <= r; i = i + 1) {
        var xi: i32;
        var yi: i32;
        if (p.axis == 0u) {
            xi = clamp(x0 + i, 0, wmax);
            yi = y0;
        } else {
            xi = x0;
            yi = clamp(y0 + i, 0, hmax);
        }
        let v = textureLoad(src, vec2<i32>(xi, yi), 0).r;
        m = min(m, v);
    }
    textureStore(dst, vec2<i32>(x0, y0), vec4<f32>(m, 0.0, 0.0, 1.0));
}
