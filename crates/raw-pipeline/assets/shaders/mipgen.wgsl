@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let sz = textureDimensions(dst);
    if (gid.x >= sz.x || gid.y >= sz.y) { return; }
    let sx = i32(gid.x * 2u);
    let sy = i32(gid.y * 2u);
    let a = textureLoad(src, vec2<i32>(sx, sy), 0);
    let b = textureLoad(src, vec2<i32>(sx + 1, sy), 0);
    let c = textureLoad(src, vec2<i32>(sx, sy + 1), 0);
    let d = textureLoad(src, vec2<i32>(sx + 1, sy + 1), 0);
    textureStore(dst, vec2<i32>(i32(gid.x), i32(gid.y)), (a + b + c + d) * 0.25);
}
