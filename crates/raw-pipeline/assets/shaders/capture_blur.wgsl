struct Params {
    size: vec2<u32>,
    axis: i32,
    mode: u32,
    levels: u32,
    half_diag: f32,
    radius: array<vec4<i32>, 8>,
    kernel: array<vec4<f32>, 128>,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var aux: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<r32float, write>;

const EPS: f32 = 1e-5;

fn sigma_level(x: i32, y: i32) -> u32 {
    if (p.levels <= 1u) { return 0u; }
    let d = vec2<f32>(f32(x) + 0.5 - 0.5 * f32(p.size.x), f32(y) + 0.5 - 0.5 * f32(p.size.y));
    let t = sqrt(dot(d, d)) / p.half_diag;
    return min(u32(floor(t * f32(p.levels - 1u) + 0.5)), p.levels - 1u);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let last_x = i32(p.size.x) - 1;
    let last_y = i32(p.size.y) - 1;
    let level = sigma_level(x, y);
    let radius = p.radius[level / 4u][level % 4u];
    let base = level * 16u;
    var acc = 0.0;
    for (var i = 0; i <= 2 * radius; i = i + 1) {
        var sx = x;
        var sy = y;
        if (p.axis == 0) {
            sx = clamp(x + i - radius, 0, last_x);
        } else {
            sy = clamp(y + i - radius, 0, last_y);
        }
        let k = base + u32(i);
        acc = acc + p.kernel[k / 4u][k % 4u] * textureLoad(src, vec2<i32>(sx, sy), 0).r;
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
