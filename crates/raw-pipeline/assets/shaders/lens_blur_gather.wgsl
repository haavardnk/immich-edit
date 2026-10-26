@group(0) @binding(1) var grid: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba32float, write>;

fn aperture_distance(dx: f32, dy: f32, blades: u32) -> f32 {
    let len = sqrt(dx * dx + dy * dy);
    if (blades == 0u || len == 0.0) {
        return len;
    }
    let sector = TAU / f32(blades);
    let theta = atan2(dy, dx) - APERTURE_ROTATION;
    let phi = theta - sector * round(theta / sector);
    return len * cos(phi) / cos(PI / f32(blades));
}

fn highlight_weight(rgb: vec3<f32>, boost: f32) -> f32 {
    let peak = max(max(rgb.r, rgb.g), rgb.b);
    return 1.0 + boost * HIGHLIGHT_GAIN * smoothstep(HIGHLIGHT_LO, HIGHLIGHT_HI, peak);
}

fn sender_reach(receiver: f32, sender: f32) -> f32 {
    if (sender < receiver) {
        return min(abs(sender), abs(receiver));
    }
    return abs(sender);
}

fn gather_weight(reach: f32, distance: f32) -> f32 {
    let coverage = clamp(reach - distance + 0.5, 0.0, 1.0);
    let area = max(reach, 0.5);
    return coverage / (area * area);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.grid_size.x || gid.y >= p.grid_size.y) {
        return;
    }
    let gw = i32(p.grid_size.x);
    let gh = i32(p.grid_size.y);
    let px = i32(gid.x);
    let py = i32(gid.y);
    let receiver = textureLoad(grid, vec2<i32>(px, py), 0).a;
    var acc = vec4<f32>(0.0);
    var total = 0.0;
    for (var dy = -p.reach; dy <= p.reach; dy = dy + 1) {
        let qy = py + dy;
        if (qy < 0 || qy >= gh) {
            continue;
        }
        for (var dx = -p.reach; dx <= p.reach; dx = dx + 1) {
            let qx = px + dx;
            if (qx < 0 || qx >= gw) {
                continue;
            }
            let q = textureLoad(grid, vec2<i32>(qx, qy), 0);
            let distance = aperture_distance(f32(dx), f32(dy), p.blades);
            let spread = sender_reach(receiver, q.a);
            var weight = gather_weight(spread, distance);
            if (weight <= 0.0) {
                continue;
            }
            weight = weight * highlight_weight(q.rgb, p.boost);
            acc = acc + vec4<f32>(q.rgb, spread) * weight;
            total = total + weight;
        }
    }
    textureStore(dst, vec2<i32>(px, py), acc / total);
}
