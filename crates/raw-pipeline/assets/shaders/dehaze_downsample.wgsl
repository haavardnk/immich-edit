struct Params {
    size_lo: vec2<u32>,
    scale: u32,
    _pad: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size_lo.x || gid.y >= p.size_lo.y) { return; }
    let pos = vec2<i32>(i32(gid.x), i32(gid.y));
    let inv = 1.0 / vec2<f32>(p.size_lo);
    let uv = (vec2<f32>(pos) + vec2<f32>(0.5)) * inv;
    let c = textureSampleLevel(src, samp, uv, 0.0).rgb;
    textureStore(dst, pos, vec4<f32>(c, 1.0));
}
