struct Params {
    size: vec2<u32>,
    read: vec2<u32>,
    write: vec2<u32>,
    scale: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let s = i32(p.scale);
    let at = vec2<i32>(gid.xy) * s + vec2<i32>(max(s / 2 - 1, 0)) + vec2<i32>(p.read);
    var c = textureLoad(src, at, 0).rgb;
    if (s > 1) {
        let right = textureLoad(src, at + vec2<i32>(1, 0), 0).rgb;
        let below = textureLoad(src, at + vec2<i32>(0, 1), 0).rgb;
        let diagonal = textureLoad(src, at + vec2<i32>(1, 1), 0).rgb;
        c = (c + right + below + diagonal) * 0.25;
    }
    textureStore(dst, vec2<i32>(gid.xy + p.write), vec4<f32>(c, 1.0));
}
