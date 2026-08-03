struct XtransParams {
    size: vec2<u32>,
    pattern: array<vec4<u32>, 9>,
};

const RAW_LINEAR_CEILING: f32 = 4.0;

@group(0) @binding(0) var<uniform> p: XtransParams;
@group(0) @binding(1) var<storage, read> raw_in: array<f32>;
@group(0) @binding(2) var<storage, read> green_in: array<f32>;
@group(0) @binding(3) var rgb_out: texture_storage_2d<rgba16float, write>;

fn channel_at(x: u32, y: u32) -> u32 {
    let i = (y % 6u) * 6u + (x % 6u);
    return p.pattern[i / 4u][i % 4u];
}

fn chroma(ix: i32, iy: i32, want: u32) -> f32 {
    let w = i32(p.size.x);
    let h = i32(p.size.y);
    var sum: f32 = 0.0;
    var weight: f32 = 0.0;
    for (var dy = -2; dy <= 2; dy = dy + 1) {
        for (var dx = -2; dx <= 2; dx = dx + 1) {
            let nx = ix + dx;
            let ny = iy + dy;
            if (nx < 0 || ny < 0 || nx >= w || ny >= h) { continue; }
            if (channel_at(u32(nx), u32(ny)) != want) { continue; }
            let wgt = 1.0 / f32(dx * dx + dy * dy);
            let n = u32(ny) * p.size.x + u32(nx);
            sum = sum + (raw_in[n] - green_in[n]) * wgt;
            weight = weight + wgt;
        }
    }
    if (weight > 0.0) { return sum / weight; }
    return 0.0;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let ix = i32(gid.x);
    let iy = i32(gid.y);
    let idx = gid.y * p.size.x + gid.x;

    let own = channel_at(gid.x, gid.y);
    let g = green_in[idx];
    var rgb = vec3<f32>(0.0, g, 0.0);

    if (own == 0u) {
        rgb.x = raw_in[idx];
    } else {
        rgb.x = clamp(g + chroma(ix, iy, 0u), 0.0, RAW_LINEAR_CEILING);
    }
    if (own == 2u) {
        rgb.z = raw_in[idx];
    } else {
        rgb.z = clamp(g + chroma(ix, iy, 2u), 0.0, RAW_LINEAR_CEILING);
    }

    textureStore(rgb_out, vec2<i32>(ix, iy), vec4<f32>(rgb, 1.0));
}
