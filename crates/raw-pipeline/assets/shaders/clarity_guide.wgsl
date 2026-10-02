struct ClarityGuideParams {
    size: vec2<u32>,
    radius: u32,
    mode: u32,
    eps: f32,
    weights: array<vec4<f32>, 3>,
}

@group(0) @binding(0) var<uniform> p: ClarityGuideParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba32float, write>;

fn tap(x: i32, y: i32) -> vec2<f32> {
    let v = textureLoad(src, vec2<i32>(x, y), 0);
    if (p.mode == 0u) {
        return vec2<f32>(log2(max(v.r, 1e-5)), 0.0);
    }
    return v.xy;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let r = i32(p.radius);
    let vertical = (p.mode & 1u) == 1u;
    let last_x = i32(p.size.x) - 1;
    let last_y = i32(p.size.y) - 1;
    var acc = vec2<f32>(0.0);
    for (var i = -r; i <= r; i = i + 1) {
        let k = u32(abs(i));
        let w = p.weights[k / 4u][k % 4u];
        if (vertical) {
            acc = acc + w * tap(x, clamp(y + i, 0, last_y));
        } else {
            acc = acc + w * tap(clamp(x + i, 0, last_x), y);
        }
    }
    var out = acc;
    if (p.mode == 1u) {
        out = vec2<f32>(acc.x, acc.x * acc.x);
    } else if (p.mode == 3u) {
        let v = max(acc.y - acc.x * acc.x, 0.0);
        let a = v / (v + p.eps);
        out = vec2<f32>(a, acc.x * (1.0 - a));
    }
    textureStore(dst, vec2<i32>(x, y), vec4<f32>(out, 0.0, 0.0));
}
