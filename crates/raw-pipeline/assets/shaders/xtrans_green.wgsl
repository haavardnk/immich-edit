struct XtransParams {
    size: vec2<u32>,
    pattern: array<vec4<u32>, 9>,
};

const RAW_LINEAR_CEILING: f32 = 4.0;

@group(0) @binding(0) var<uniform> p: XtransParams;
@group(0) @binding(1) var<storage, read> raw_in: array<f32>;
@group(0) @binding(2) var<storage, read_write> green_out: array<f32>;

fn channel_at(x: u32, y: u32) -> u32 {
    let i = (y % 6u) * 6u + (x % 6u);
    return p.pattern[i / 4u][i % 4u];
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let idx = gid.y * p.size.x + gid.x;
    let own = raw_in[idx];

    if (channel_at(gid.x, gid.y) == 1u) {
        green_out[idx] = own;
        return;
    }

    let ix = i32(gid.x);
    let iy = i32(gid.y);
    let w = i32(p.size.x);
    let h = i32(p.size.y);

    var sum: f32 = 0.0;
    var weight: f32 = 0.0;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            if (dx == 0 && dy == 0) { continue; }
            let nx = ix + dx;
            let ny = iy + dy;
            if (nx < 0 || ny < 0 || nx >= w || ny >= h) { continue; }
            if (channel_at(u32(nx), u32(ny)) != 1u) { continue; }
            let wgt = select(0.5, 1.0, dx == 0 || dy == 0);
            sum = sum + raw_in[u32(ny) * p.size.x + u32(nx)] * wgt;
            weight = weight + wgt;
        }
    }

    if (weight > 0.0) {
        green_out[idx] = clamp(sum / weight, 0.0, RAW_LINEAR_CEILING);
    } else {
        green_out[idx] = own;
    }
}
