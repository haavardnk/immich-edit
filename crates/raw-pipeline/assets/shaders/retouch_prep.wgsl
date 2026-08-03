// color-space: linear scene-referred Rgba16Float in/out; patch-local source and residual
struct Params {
    img: vec2<u32>,
    bbox_origin: vec2<u32>,
    bbox_size: vec2<u32>,
    n_points: u32,
    mode: u32,
    offset: vec2<f32>,
    radius: f32,
    hardness: f32,
    opacity: f32,
    sigma: f32,
    dir: u32,
    pad: u32,
};

@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var src_tex: texture_2d<f32>;
@group(0) @binding(2) var patch_src: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var patch_res: texture_storage_2d<rgba16float, write>;

fn cr_weights(t: f32) -> vec4<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    return vec4<f32>(
        -0.5 * t3 + t2 - 0.5 * t,
        1.5 * t3 - 2.5 * t2 + 1.0,
        -1.5 * t3 + 2.0 * t2 + 0.5 * t,
        0.5 * t3 - 0.5 * t2,
    );
}

fn load_clamped(x: i32, y: i32) -> vec3<f32> {
    let cx = clamp(x, 0, i32(p.img.x) - 1);
    let cy = clamp(y, 0, i32(p.img.y) - 1);
    return textureLoad(src_tex, vec2<i32>(cx, cy), 0).rgb;
}

fn sample_bicubic(x: f32, y: f32) -> vec3<f32> {
    let fx = floor(x);
    let fy = floor(y);
    let wx = cr_weights(x - fx);
    let wy = cr_weights(y - fy);
    let ix0 = i32(fx) - 1;
    let iy0 = i32(fy) - 1;
    var acc = vec3<f32>(0.0);
    for (var j = 0; j < 4; j = j + 1) {
        let yy = iy0 + j;
        var row = wx.x * load_clamped(ix0, yy);
        row = row + wx.y * load_clamped(ix0 + 1, yy);
        row = row + wx.z * load_clamped(ix0 + 2, yy);
        row = row + wx.w * load_clamped(ix0 + 3, yy);
        var wj = wy.x;
        if (j == 1) { wj = wy.y; } else if (j == 2) { wj = wy.z; } else if (j == 3) { wj = wy.w; }
        acc = acc + wj * row;
    }
    return acc;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.bbox_size.x || gid.y >= p.bbox_size.y) { return; }
    let gx = i32(p.bbox_origin.x + gid.x);
    let gy = i32(p.bbox_origin.y + gid.y);
    let s = sample_bicubic(f32(gx) + p.offset.x, f32(gy) + p.offset.y);
    let d = load_clamped(gx, gy);
    let c = vec2<i32>(i32(gid.x), i32(gid.y));
    textureStore(patch_src, c, vec4<f32>(s, 1.0));
    textureStore(patch_res, c, vec4<f32>(d - s, 1.0));
}
