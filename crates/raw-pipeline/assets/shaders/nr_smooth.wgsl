struct SmoothParams {
    size: vec2<u32>,
    _pad: vec2<u32>,
    s: f32,
    alpha: f32,
    _pad2: vec2<f32>,
}

@group(0) @binding(0) var<uniform> p: SmoothParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var chroma: texture_2d<f32>;
@group(0) @binding(3) var dst: texture_storage_2d<rgba16float, write>;

const KR: f32 = 0.2126;
const KG: f32 = 0.7152;
const KB: f32 = 0.0722;
const PB_DEN: f32 = 1.8556;
const PR_DEN: f32 = 1.5748;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let cx = i32(gid.x);
    let cy = i32(gid.y);
    let rgb = textureLoad(src, vec2<i32>(cx, cy), 0).rgb;
    let y_c = KR * rgb.r + KG * rgb.g + KB * rgb.b;
    let pb_orig = (rgb.b - y_c) / PB_DEN;
    let pr_orig = (rgb.r - y_c) / PR_DEN;

    let den = textureLoad(chroma, vec2<i32>(cx, cy), 0).rg;
    var pb_den = den.x;
    var pr_den = den.y;

    if (p.s > 0.0) {
        let y0 = max(cy - 1, 0);
        let y1 = min(cy + 1, i32(p.size.y) - 1);
        let x0 = max(cx - 1, 0);
        let x1 = min(cx + 1, i32(p.size.x) - 1);
        var sum_pb = 0.0;
        var sum_pr = 0.0;
        var cnt = 0.0;
        for (var sy = y0; sy <= y1; sy = sy + 1) {
            for (var sx = x0; sx <= x1; sx = sx + 1) {
                let n = textureLoad(chroma, vec2<i32>(sx, sy), 0).rg;
                sum_pb = sum_pb + n.x;
                sum_pr = sum_pr + n.y;
                cnt = cnt + 1.0;
            }
        }
        pb_den = pb_den + (sum_pb / cnt - pb_den) * p.s;
        pr_den = pr_den + (sum_pr / cnt - pr_den) * p.s;
    }

    let pb_new = pb_orig + (pb_den - pb_orig) * p.alpha;
    let pr_new = pr_orig + (pr_den - pr_orig) * p.alpha;
    let r_out = y_c + PR_DEN * pr_new;
    let b_out = y_c + PB_DEN * pb_new;
    let g_out = (y_c - KR * r_out - KB * b_out) / KG;
    textureStore(dst, vec2<i32>(cx, cy), vec4<f32>(r_out, g_out, b_out, 1.0));
}
