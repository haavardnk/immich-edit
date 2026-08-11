struct NrParams {
    size: vec2<u32>,
    radius: u32,
    stage: u32,
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
    return textureLoad(src, vec2<i32>(x, y), 0).rgb;
}

fn luma_of(rgb: vec3<f32>) -> f32 {
    return KR * rgb.r + KG * rgb.g + KB * rgb.b;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let cx = i32(gid.x);
    let cy = i32(gid.y);
    let center_rgb = load_rgb(cx, cy);
    let c_y = luma_of(center_rgb);
    let c_pb = (center_rgb.b - c_y) / PB_DEN;
    let c_pr = (center_rgb.r - c_y) / PR_DEN;

    let r = i32(p.radius);
    let y0 = max(cy - r, 0);
    let y1 = min(cy + r, i32(p.size.y) - 1);
    let x0 = max(cx - r, 0);
    let x1 = min(cx + r, i32(p.size.x) - 1);
    let inv_2sr = select(p.inv_2sr_chroma, p.inv_2sr_luma, p.stage == 0u);

    var wsum: f32 = 0.0;
    var acc_l: f32 = 0.0;
    var acc_pb: f32 = 0.0;
    var acc_pr: f32 = 0.0;

    for (var sy = y0; sy <= y1; sy = sy + 1) {
        for (var sx = x0; sx <= x1; sx = sx + 1) {
            let rgb = load_rgb(sx, sy);
            let yv = luma_of(rgb);
            let pb = (rgb.b - yv) / PB_DEN;
            let pr = (rgb.r - yv) / PR_DEN;
            let dx = f32(sx - cx);
            let dy = f32(sy - cy);
            let spatial = -(dx * dx + dy * dy) * p.inv_2ss;
            var range = 0.0;
            if (p.stage == 0u) {
                let dl = yv - c_y;
                range = dl * dl;
            } else {
                let dpb = pb - c_pb;
                let dpr = pr - c_pr;
                range = dpb * dpb + dpr * dpr;
            }
            let wgt = exp(spatial - range * inv_2sr);
            wsum = wsum + wgt;
            acc_l = acc_l + wgt * yv;
            acc_pb = acc_pb + wgt * pb;
            acc_pr = acc_pr + wgt * pr;
        }
    }

    if (p.stage == 0u) {
        var y_den = c_y;
        if (wsum > 0.0) { y_den = acc_l / wsum; }
        let alpha = p.alpha_luma * (1.0 - p.contrast);
        let y_new = c_y + (y_den - c_y) * alpha;
        var scale = 1.0;
        if (c_y > 1e-6) { scale = y_new / c_y; }
        textureStore(dst, vec2<i32>(cx, cy), vec4<f32>(center_rgb * scale, 1.0));
        return;
    }

    var pb_den = c_pb;
    var pr_den = c_pr;
    if (wsum > 0.0) {
        pb_den = acc_pb / wsum;
        pr_den = acc_pr / wsum;
    }
    textureStore(dst, vec2<i32>(cx, cy), vec4<f32>(pb_den, pr_den, 0.0, 1.0));
}
