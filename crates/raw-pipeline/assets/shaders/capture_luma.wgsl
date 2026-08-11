struct Params {
    size: vec2<u32>,
    _pad: vec2<u32>,
}

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<r32float, write>;
@group(0) @binding(3) var est: texture_storage_2d<r32float, write>;

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let c = vec2<i32>(i32(gid.x), i32(gid.y));
    let rgb = textureLoad(src, c, 0).rgb;
    let y = max(KR * rgb.r + KG * rgb.g + KB * rgb.b, 0.0);
    textureStore(dst, c, vec4<f32>(y, 0.0, 0.0, 0.0));
    textureStore(est, c, vec4<f32>(y, 0.0, 0.0, 0.0));
}
