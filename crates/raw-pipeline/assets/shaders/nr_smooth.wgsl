struct SmoothParams {
    size: vec2<u32>,
    _pad: vec2<u32>,
    s: f32,
    _pad2: vec3<f32>,
}

@group(0) @binding(0) var<uniform> p: SmoothParams;
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

fn chroma_of(rgb: vec3<f32>) -> vec3<f32> {
    let y = KR * rgb.r + KG * rgb.g + KB * rgb.b;
    let pb = (rgb.b - y) / PB_DEN;
    let pr = (rgb.r - y) / PR_DEN;
    return vec3<f32>(y, pb, pr);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let cx = i32(gid.x);
    let cy = i32(gid.y);
    let center = load_rgb(cx, cy);
    let cypbpr = chroma_of(center);
    let y_c = cypbpr.x;
    let pb_c = cypbpr.y;
    let pr_c = cypbpr.z;

    var sum_pb: f32 = 0.0;
    var sum_pr: f32 = 0.0;
    var cnt: f32 = 0.0;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let n = chroma_of(load_rgb(cx + dx, cy + dy));
            sum_pb = sum_pb + n.y;
            sum_pr = sum_pr + n.z;
            cnt = cnt + 1.0;
        }
    }
    let pb_b = sum_pb / cnt;
    let pr_b = sum_pr / cnt;
    let pb_new = pb_c + (pb_b - pb_c) * p.s;
    let pr_new = pr_c + (pr_b - pr_c) * p.s;
    let r_out = y_c + PR_DEN * pr_new;
    let b_out = y_c + PB_DEN * pb_new;
    let g_out = (y_c - KR * r_out - KB * b_out) / KG;
    textureStore(dst, vec2<i32>(cx, cy), vec4<f32>(r_out, g_out, b_out, 1.0));
}
