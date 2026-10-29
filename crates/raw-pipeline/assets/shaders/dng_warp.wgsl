struct DngWarpParams {
    size: vec2<u32>,
    center: vec2<f32>,
    radius: f32,
    split: u32,
    dst_origin: vec2<u32>,
    radial: array<vec4<f32>, 3>,
    tangential: array<vec4<f32>, 3>,
    src_origin: vec2<i32>,
};

@group(0) @binding(0) var<uniform> p: DngWarpParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

fn source_px(plane: u32, pos: vec2<f32>) -> vec2<f32> {
    let d = (pos - p.center) / p.radius;
    let r2 = d.x * d.x + d.y * d.y;
    let k = p.radial[plane];
    let t = p.tangential[plane];
    let f = k.x + r2 * (k.y + r2 * (k.z + r2 * k.w));
    let tx = t.x * 2.0 * d.x * d.y + t.y * (r2 + 2.0 * d.x * d.x);
    let ty = t.y * 2.0 * d.x * d.y + t.x * (r2 + 2.0 * d.y * d.y);
    return p.center + p.radius * (f * d + vec2<f32>(tx, ty)) - vec2<f32>(0.5);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let px = gid.xy + p.dst_origin;
    let pos = vec2<f32>(f32(px.x) + 0.5, f32(px.y) + 0.5);
    let gid_i = vec2<i32>(i32(gid.x), i32(gid.y));
    if (p.split == 0u) {
        let rgb = bicubic_rgb(source_px(0u, pos));
        textureStore(dst, gid_i, vec4<f32>(max(rgb, vec3<f32>(0.0)), 1.0));
        return;
    }
    let red = bicubic_rgb(source_px(0u, pos)).r;
    let green = bicubic_rgb(source_px(1u, pos)).g;
    let blue = bicubic_rgb(source_px(2u, pos)).b;
    textureStore(dst, gid_i, vec4<f32>(max(vec3<f32>(red, green, blue), vec3<f32>(0.0)), 1.0));
}
