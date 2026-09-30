struct NrSizeParams {
    size: vec2<u32>,
}

@group(0) @binding(0) var<uniform> p: NrSizeParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<r32float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let c = vec2<i32>(gid.xy);
    textureStore(dst, c, vec4<f32>(nr_luma(textureLoad(src, c, 0).rgb), 0.0, 0.0, 0.0));
}
