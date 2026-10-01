struct NrSizeParams {
    size: vec2<u32>,
}

@group(0) @binding(0) var<uniform> p: NrSizeParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var acc: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

fn chroma_scale(y0: f32, y1: f32) -> f32 {
    if (y0 > 1e-6 && y1 > 0.0) {
        return clamp(sqrt(y1 / y0), 0.5, 2.0);
    }
    return 1.0;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let c = vec2<i32>(gid.xy);
    let px = textureLoad(src, c, 0);
    let y0 = nr_luma(px.rgb);
    let y1 = y0 + textureLoad(acc, c, 0).r;
    let rgb = vec3<f32>(y1) + (px.rgb - vec3<f32>(y0)) * chroma_scale(y0, y1);
    textureStore(dst, c, vec4<f32>(rgb, px.a));
}
