struct SharpenParams {
    sharpen: vec4<f32>,
    dims_flags: vec4<u32>,
};

@group(0) @binding(0) var<uniform> p: SharpenParams;
@group(0) @binding(1) var src_lin: texture_2d<f32>;
@group(0) @binding(2) var src_blur: texture_2d<f32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var sharpened_lin: texture_storage_2d<rgba16float, write>;

fn luma(c: vec3<f32>) -> f32 {
    return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
}

fn smoothstep_f(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / max(edge1 - edge0, 1e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = p.dims_flags.x;
    let height = p.dims_flags.y;
    let use_mask = p.dims_flags.z;
    let preview_mode = p.dims_flags.w;
    if (gid.x >= width || gid.y >= height) { return; }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let max_x = i32(width) - 1;
    let max_y = i32(height) - 1;

    let amount = p.sharpen.x;
    let detail_weight = p.sharpen.y;
    let masking_thresh = p.sharpen.z;
    let masking_softness = p.sharpen.w;

    let orig = textureLoad(src_lin, vec2<i32>(x, y), 0).rgb;
    let blur = textureLoad(src_blur, vec2<i32>(x, y), 0).rgb;
    let hp = orig - blur;

    var mask = 1.0;
    if (use_mask == 1u) {
        let xm = max(x - 1, 0);
        let xp = min(x + 1, max_x);
        let ym = max(y - 1, 0);
        let yp = min(y + 1, max_y);
        let l00 = luma(textureLoad(src_blur, vec2<i32>(xm, ym), 0).rgb);
        let l10 = luma(textureLoad(src_blur, vec2<i32>(x, ym), 0).rgb);
        let l20 = luma(textureLoad(src_blur, vec2<i32>(xp, ym), 0).rgb);
        let l01 = luma(textureLoad(src_blur, vec2<i32>(xm, y), 0).rgb);
        let l21 = luma(textureLoad(src_blur, vec2<i32>(xp, y), 0).rgb);
        let l02 = luma(textureLoad(src_blur, vec2<i32>(xm, yp), 0).rgb);
        let l12 = luma(textureLoad(src_blur, vec2<i32>(x, yp), 0).rgb);
        let l22 = luma(textureLoad(src_blur, vec2<i32>(xp, yp), 0).rgb);
        let gx = (l20 + 2.0 * l21 + l22) - (l00 + 2.0 * l01 + l02);
        let gy = (l02 + 2.0 * l12 + l22) - (l00 + 2.0 * l10 + l20);
        let mag = sqrt(gx * gx + gy * gy);
        mask = smoothstep_f(masking_thresh, masking_thresh + masking_softness, mag);
    }

    let strength = (amount / 50.0) * detail_weight * mask;
    let lin = orig + hp * strength;

    textureStore(sharpened_lin, vec2<i32>(x, y), vec4<f32>(lin, 1.0));

    if (preview_mode == 1u) {
        textureStore(out_tex, vec2<i32>(x, y), vec4<f32>(mask, mask, mask, 1.0));
    } else if (preview_mode == 2u) {
        let lb = luma(blur);
        textureStore(out_tex, vec2<i32>(x, y), vec4<f32>(lb, lb, lb, 1.0));
    } else if (preview_mode == 3u) {
        let lr = luma(orig);
        let lb = luma(blur);
        let d = clamp(8.0 * detail_weight * abs(lr - lb), 0.0, 1.0);
        textureStore(out_tex, vec2<i32>(x, y), vec4<f32>(d, d, d, 1.0));
    }
}
