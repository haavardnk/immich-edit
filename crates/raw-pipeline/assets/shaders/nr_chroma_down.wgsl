struct NrSizeParams {
    size: vec2<u32>,
}

@group(0) @binding(0) var<uniform> p: NrSizeParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var cur: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var base: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let lo = (p.size + 1u) / 2u;
    if (gid.x >= lo.x || gid.y >= lo.y) { return; }
    let last = vec2<i32>(p.size) - 1;
    let x = vec2<i32>(min(i32(2u * gid.x), last.x), min(i32(2u * gid.x + 1u), last.x));
    let y = vec2<i32>(min(i32(2u * gid.y), last.y), min(i32(2u * gid.y + 1u), last.y));
    let top = textureLoad(src, vec2<i32>(x.x, y.x), 0).rgb + textureLoad(src, vec2<i32>(x.y, y.x), 0).rgb;
    let bottom = textureLoad(src, vec2<i32>(x.x, y.y), 0).rgb + textureLoad(src, vec2<i32>(x.y, y.y), 0).rgb;
    let v = vec4<f32>(to_chroma((top + bottom) * 0.25), 0.0);
    let c = vec2<i32>(gid.xy);
    textureStore(cur, c, v);
    textureStore(base, c, v);
}
