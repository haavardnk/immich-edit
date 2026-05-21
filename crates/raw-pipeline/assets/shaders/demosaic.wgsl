struct DemosaicParams {
    size: vec2<u32>,
    cfa: vec4<u32>,
};

@group(0) @binding(0) var<uniform> p: DemosaicParams;
@group(0) @binding(1) var<storage, read> raw_in: array<f32>;
@group(0) @binding(2) var rgb_out: texture_storage_2d<rgba16float, write>;

fn cfa_at(x: u32, y: u32) -> u32 {
    return p.cfa[(y & 1u) * 2u + (x & 1u)];
}

fn fetch(ix: i32, iy: i32) -> f32 {
    let xc = clamp(ix, 0, i32(p.size.x) - 1);
    let yc = clamp(iy, 0, i32(p.size.y) - 1);
    let idx = u32(yc) * p.size.x + u32(xc);
    return raw_in[idx];
}

fn avg_color(ix: i32, iy: i32, want: u32) -> f32 {
    var sum: f32 = 0.0;
    var n: f32 = 0.0;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let x = ix + dx;
            let y = iy + dy;
            let xc = clamp(x, 0, i32(p.size.x) - 1);
            let yc = clamp(y, 0, i32(p.size.y) - 1);
            if (cfa_at(u32(xc), u32(yc)) == want) {
                sum = sum + fetch(xc, yc);
                n = n + 1.0;
            }
        }
    }
    if (n == 0.0) { return 0.0; }
    return sum / n;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let ix = i32(gid.x);
    let iy = i32(gid.y);
    let own_c = cfa_at(gid.x, gid.y);
    let own_v = fetch(ix, iy);

    var r: f32;
    var g: f32;
    var b: f32;
    if (own_c == 0u) {
        r = own_v;
        g = avg_color(ix, iy, 1u);
        b = avg_color(ix, iy, 2u);
    } else if (own_c == 2u) {
        r = avg_color(ix, iy, 0u);
        g = avg_color(ix, iy, 1u);
        b = own_v;
    } else {
        r = avg_color(ix, iy, 0u);
        g = own_v;
        b = avg_color(ix, iy, 2u);
    }
    textureStore(rgb_out, vec2<i32>(ix, iy), vec4<f32>(r, g, b, 1.0));
}
