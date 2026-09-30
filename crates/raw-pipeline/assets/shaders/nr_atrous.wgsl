struct NrAtrousParams {
    size: vec2<u32>,
    step: u32,
    axis: u32,
}

@group(0) @binding(0) var<uniform> p: NrAtrousParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let c = vec2<i32>(gid.xy);
    let last = vec2<i32>(p.size) - 1;
    let s = i32(p.step);
    let dir = select(vec2<i32>(s, 0), vec2<i32>(0, s), p.axis == 1u);
    let centre = textureLoad(src, c, 0);
    let m1 = textureLoad(src, clamp(c - dir, vec2<i32>(0), last), 0);
    let p1 = textureLoad(src, clamp(c + dir, vec2<i32>(0), last), 0);
    let m2 = textureLoad(src, clamp(c - 2 * dir, vec2<i32>(0), last), 0);
    let p2 = textureLoad(src, clamp(c + 2 * dir, vec2<i32>(0), last), 0);
    textureStore(dst, c, B3_0 * centre + B3_1 * (m1 + p1) + B3_2 * (m2 + p2));
}
