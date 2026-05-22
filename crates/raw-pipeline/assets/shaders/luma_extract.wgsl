@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let sz = textureDimensions(dst);
    if (gid.x >= sz.x || gid.y >= sz.y) { return; }
    let rgb = textureLoad(src, vec2<i32>(i32(gid.x), i32(gid.y)), 0).rgb;
    let y = 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
    textureStore(dst, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(y, 0.0, 0.0, 1.0));
}
