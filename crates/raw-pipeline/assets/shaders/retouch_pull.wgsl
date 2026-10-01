// color-space: linear scene-referred Rgba16Float in/out; fills the heal residual inside the stroke from the coarser level
@group(0) @binding(0) var fine: texture_2d<f32>;
@group(0) @binding(1) var coarse: texture_2d<f32>;
@group(0) @binding(2) var out_tex: texture_storage_2d<rgba16float, write>;

fn coarse_at(c: vec2<i32>, cmax: vec2<i32>) -> vec3<f32> {
    let v = textureLoad(coarse, clamp(c, vec2<i32>(0), cmax), 0);
    return v.rgb / max(v.a, 1e-8);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let size = textureDimensions(out_tex);
    if (gid.x >= size.x || gid.y >= size.y) { return; }
    let cmax = vec2<i32>(textureDimensions(coarse)) - 1;
    let u = (vec2<f32>(gid.xy) + 0.5) * 0.5 - 0.5;
    let t = u - floor(u);
    let i0 = vec2<i32>(floor(u));
    let top = mix(coarse_at(i0, cmax), coarse_at(i0 + vec2<i32>(1, 0), cmax), t.x);
    let bottom = mix(coarse_at(i0 + vec2<i32>(0, 1), cmax), coarse_at(i0 + vec2<i32>(1, 1), cmax), t.x);
    let f = textureLoad(fine, vec2<i32>(gid.xy), 0);
    textureStore(out_tex, vec2<i32>(gid.xy), vec4<f32>(f.rgb + (1.0 - f.a) * mix(top, bottom, t.y), 1.0));
}
