struct DenoiseBlendParams {
    size: vec2<u32>,
    strength: f32,
    detail: f32,
    luma: vec3<f32>,
    neutral: vec3<f32>,
};

@group(0) @binding(0) var<uniform> p: DenoiseBlendParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var plane: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) {
        return;
    }
    let at = vec2<i32>(gid.xy);
    let original = textureLoad(src, at, 0);
    let denoised = textureLoad(plane, at, 0).rgb;
    let grain = p.strength * p.detail * dot(p.luma, original.rgb - denoised);
    let rgb = original.rgb + p.strength * (denoised - original.rgb) + grain * p.neutral;
    textureStore(dst, at, vec4<f32>(rgb, original.a));
}
