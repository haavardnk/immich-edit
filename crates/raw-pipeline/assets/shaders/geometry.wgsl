// color-space: none, coordinate mapping only
fn geom_display_to_oriented(
    crop: vec4<f32>,
    g2: vec4<f32>,
    g3: vec4<f32>,
    uv: vec2<f32>,
) -> vec2<f32> {
    let bx_rel = crop.x + uv.x * crop.z;
    let by_rel = crop.y + uv.y * crop.w;
    let cx_px = (bx_rel - 0.5) * g2.z;
    let cy_px = (by_rel - 0.5) * g2.w;
    let sx_px = cx_px * g2.x + cy_px * g2.y;
    let sy_px = -cx_px * g2.y + cy_px * g2.x;
    return vec2<f32>(sx_px / g3.x + 0.5, sy_px / g3.y + 0.5);
}

fn geom_ortho_inverse(flags: vec4<u32>, uv: vec2<f32>) -> vec2<f32> {
    var cu = uv.x;
    var cv = uv.y;
    if (flags.y == 1u) { cu = 1.0 - cu; }
    if (flags.z == 1u) { cv = 1.0 - cv; }
    let rot = flags.x;
    if (rot == 90u) { return vec2<f32>(cv, 1.0 - cu); }
    if (rot == 180u) { return vec2<f32>(1.0 - cu, 1.0 - cv); }
    if (rot == 270u) { return vec2<f32>(1.0 - cv, cu); }
    return vec2<f32>(cu, cv);
}
