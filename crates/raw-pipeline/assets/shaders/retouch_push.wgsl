// color-space: linear scene-referred Rgba16Float in/out; premultiplied heal residual, confidence in alpha
@group(0) @binding(0) var fine: texture_2d<f32>;
@group(0) @binding(1) var coarse: texture_storage_2d<rgba16float, write>;

fn tent(i: i32) -> f32 {
    return select(0.375, 0.125, i == 0 || i == 3);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(coarse);
    if (gid.x >= size.x || gid.y >= size.y) { return; }
    let fine_max = vec2<i32>(textureDimensions(fine)) - 1;
    var acc = vec4<f32>(0.0);
    for (var j = 0; j < 4; j = j + 1) {
        let fy = clamp(2 * i32(gid.y) - 1 + j, 0, fine_max.y);
        for (var i = 0; i < 4; i = i + 1) {
            let fx = clamp(2 * i32(gid.x) - 1 + i, 0, fine_max.x);
            acc = acc + tent(i) * tent(j) * textureLoad(fine, vec2<i32>(fx, fy), 0);
        }
    }
    textureStore(coarse, vec2<i32>(gid.xy), acc);
}
