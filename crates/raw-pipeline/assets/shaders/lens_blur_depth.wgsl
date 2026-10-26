@group(0) @binding(2) var depth: texture_2d<f32>;

fn depth_at(uv: vec2<f32>) -> f32 {
    let dims = vec2<i32>(textureDimensions(depth));
    let fx = max(clamp(uv.x, 0.0, 1.0) * f32(dims.x) - 0.5, 0.0);
    let fy = max(clamp(uv.y, 0.0, 1.0) * f32(dims.y) - 0.5, 0.0);
    let x0 = clamp(i32(floor(fx)), 0, dims.x - 1);
    let y0 = clamp(i32(floor(fy)), 0, dims.y - 1);
    let x1 = min(x0 + 1, dims.x - 1);
    let y1 = min(y0 + 1, dims.y - 1);
    let tx = fx - f32(x0);
    let ty = fy - f32(y0);
    let a = textureLoad(depth, vec2<i32>(x0, y0), 0).r;
    let b = textureLoad(depth, vec2<i32>(x1, y0), 0).r;
    let c = textureLoad(depth, vec2<i32>(x0, y1), 0).r;
    let d = textureLoad(depth, vec2<i32>(x1, y1), 0).r;
    let top = a + (b - a) * tx;
    let bot = c + (d - c) * tx;
    return top + (bot - top) * ty;
}

fn signed_coc_at(o: vec2<u32>) -> f32 {
    let od = vec2<f32>(oriented_dims());
    let uv = (vec2<f32>(o) + vec2<f32>(0.5)) / od;
    return p.radius * coc_table(depth_at(lens_mask_to_scene_uv(p.lens, uv, od)));
}
