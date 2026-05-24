struct NrParams {
    size: vec2<u32>,
    radius: u32,
    _pad: u32,
    inv_2ss: f32,
    inv_2sr_luma: f32,
    inv_2sr_chroma: f32,
    alpha_luma: f32,
    alpha_chroma: f32,
    contrast: f32,
    _pad2: vec2<f32>,
}

@group(0) @binding(0) var<uniform> p: NrParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;
const PB_DEN: f32 = 1.8556;
const PR_DEN: f32 = 1.5748;

fn load_rgb(x: i32, y: i32) -> vec3<f32> {
    let dim = textureDimensions(src);
    let ix = clamp(x, 0, i32(dim.x) - 1);
    let iy = clamp(y, 0, i32(dim.y) - 1);
    return textureLoad(src, vec2<i32>(ix, iy), 0).rgb;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let cx = i32(gid.x);
    let cy = i32(gid.y);
    let center_rgb = load_rgb(cx, cy);
    let cy_y = KR * center_rgb.r + KG * center_rgb.g + KB * center_rgb.b;
    let cy_pb = (center_rgb.b - cy_y) / PB_DEN;
    let cy_pr = (center_rgb.r - cy_y) / PR_DEN;

    let r = i32(p.radius);
    var wsum_l: f32 = 0.0;
    var acc_l: f32 = 0.0;
    var wsum_c: f32 = 0.0;
    var acc_pb: f32 = 0.0;
    var acc_pr: f32 = 0.0;

    for (var dy = -r; dy <= r; dy = dy + 1) {
        for (var dx = -r; dx <= r; dx = dx + 1) {
            let rgb = load_rgb(cx + dx, cy + dy);
            let yv = KR * rgb.r + KG * rgb.g + KB * rgb.b;
            let pb = (rgb.b - yv) / PB_DEN;
            let pr = (rgb.r - yv) / PR_DEN;
            let s2 = f32(dx * dx + dy * dy);
            let spatial = -s2 * p.inv_2ss;
            let dl = yv - cy_y;
            let wl = exp(spatial - dl * dl * p.inv_2sr_luma);
            wsum_l = wsum_l + wl;
            acc_l = acc_l + wl * yv;
            let dpb = pb - cy_pb;
            let dpr = pr - cy_pr;
            let dc2 = dpb * dpb + dpr * dpr;
            let wc = exp(spatial - dc2 * p.inv_2sr_chroma);
            wsum_c = wsum_c + wc;
            acc_pb = acc_pb + wc * pb;
            acc_pr = acc_pr + wc * pr;
        }
    }

    var y_den = cy_y;
    if (wsum_l > 0.0) { y_den = acc_l / wsum_l; }
    var pb_den2 = cy_pb;
    var pr_den2 = cy_pr;
    if (wsum_c > 0.0) {
        pb_den2 = acc_pb / wsum_c;
        pr_den2 = acc_pr / wsum_c;
    }

    let alpha_l = p.alpha_luma * (1.0 - p.contrast);
    let y_new = cy_y + (y_den - cy_y) * alpha_l;
    let pb_new = cy_pb + (pb_den2 - cy_pb) * p.alpha_chroma;
    let pr_new = cy_pr + (pr_den2 - cy_pr) * p.alpha_chroma;

    let r_out = y_new + PR_DEN * pr_new;
    let b_out = y_new + PB_DEN * pb_new;
    let g_out = (y_new - KR * r_out - KB * b_out) / KG;
    textureStore(dst, vec2<i32>(cx, cy), vec4<f32>(r_out, g_out, b_out, 1.0));
}
