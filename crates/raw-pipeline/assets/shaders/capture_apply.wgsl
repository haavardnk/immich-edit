struct Params {
    size: vec2<u32>,
    levels: u32,
    half_diag: f32,
    radius: array<vec4<i32>, 8>,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var luma: texture_2d<f32>;
@group(0) @binding(3) var est: texture_2d<f32>;
@group(0) @binding(4) var dst: texture_storage_2d<rgba16float, write>;

const EPS: f32 = 1e-5;
const CLIP_KNEE: f32 = 0.90;
const CLIP_LIMIT: f32 = 0.98;
const SHADOW_FLOOR: f32 = 0.002;
const SHADOW_KNEE: f32 = 0.02;
const CONTRAST_FLOOR: f32 = 0.010;
const CONTRAST_KNEE: f32 = 0.045;

fn lum(x: i32, y: i32) -> f32 {
    let last_x = i32(p.size.x) - 1;
    let last_y = i32(p.size.y) - 1;
    return textureLoad(luma, vec2<i32>(clamp(x, 0, last_x), clamp(y, 0, last_y)), 0).r;
}

fn light(x: i32, y: i32) -> f32 {
    let v = lum(x, y);
    return select(pow(v, 1.0 / 3.0), 0.0, v <= 0.0);
}

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
    let c = vec2<i32>(x, y);
    let px = textureLoad(src, c, 0);
    let old = lum(x, y);
    if (old <= EPS) {
        textureStore(dst, c, px);
        return;
    }
    let gx = light(x + 1, y) - light(x - 1, y);
    let gy = light(x, y + 1) - light(x, y - 1);
    let grad = sqrt(gx * gx + gy * gy);
    let peak = max(px.r, max(px.g, px.b));
    let blend = smoothstep(CONTRAST_FLOOR, CONTRAST_KNEE, grad)
        * smoothstep(SHADOW_FLOOR, SHADOW_KNEE, old)
        * (1.0 - smoothstep(CLIP_KNEE, CLIP_LIMIT, peak));
    let level = sigma_level(x, y);
    let radius = p.radius[level / 4u][level % 4u];
    var lo = old;
    var hi = old;
    for (var dy = -radius; dy <= radius; dy = dy + 1) {
        for (var dx = -radius; dx <= radius; dx = dx + 1) {
            let v = lum(x + dx, y + dy);
            lo = min(lo, v);
            hi = max(hi, v);
        }
    }
    let e = textureLoad(est, c, 0).r;
    let corrected = clamp(old + (e - old) * blend, lo, hi);
    textureStore(dst, c, vec4<f32>(px.rgb * (corrected / old), px.a));
}
