struct CombineParams {
    amount: f32,
    detail_weight: f32,
    masking_thresh: f32,
    masking_softness: f32,
    width: u32,
    height: u32,
    use_mask: u32,
    preview_mode: u32,
};

@group(0) @binding(0) var<uniform> p: CombineParams;
@group(0) @binding(1) var src_lin: texture_2d<f32>;
@group(0) @binding(2) var src_blur: texture_2d<f32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;

fn soft_clip_high(v: f32) -> f32 {
    let knee: f32 = 0.95;
    if (v <= knee) { return v; }
    let headroom: f32 = 1.0 - knee;
    let excess: f32 = v - knee;
    return knee + headroom * (excess / (excess + headroom));
}

fn default_tone(v: f32) -> f32 {
    var lin: f32;
    if (v <= 0.0) { lin = 0.0; } else { lin = soft_clip_high(v); }
    var srgb: f32;
    if (lin <= 0.003130808) {
        srgb = 12.92 * lin;
    } else {
        srgb = 1.055 * pow(lin, 1.0 / 2.4) - 0.055;
    }
    let s = srgb * srgb * (3.0 - 2.0 * srgb);
    return srgb + (s - srgb) * 0.15;
}

fn luma(c: vec3<f32>) -> f32 {
    return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
}

fn smoothstep_f(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = clamp((x - edge0) / max(edge1 - edge0, 1e-6), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.width || gid.y >= p.height) { return; }
    let x = i32(gid.x);
    let y = i32(gid.y);
    let max_x = i32(p.width) - 1;
    let max_y = i32(p.height) - 1;

    let orig = textureLoad(src_lin, vec2<i32>(x, y), 0).rgb;
    let blur = textureLoad(src_blur, vec2<i32>(x, y), 0).rgb;
    let hp = orig - blur;

    var mask = 1.0;
    if (p.use_mask == 1u) {
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
        mask = smoothstep_f(p.masking_thresh, p.masking_thresh + p.masking_softness, mag);
    }

    let strength = (p.amount / 50.0) * p.detail_weight * mask;
    let outc_lin = orig + hp * strength;
    var outc = vec3<f32>(default_tone(outc_lin.r), default_tone(outc_lin.g), default_tone(outc_lin.b));
    if (p.preview_mode == 1u) {
        outc = vec3<f32>(mask, mask, mask);
    } else if (p.preview_mode == 2u) {
        let lb = luma(blur);
        outc = vec3<f32>(lb, lb, lb);
    } else if (p.preview_mode == 3u) {
        let lr = luma(orig);
        let lb = luma(blur);
        let d = clamp(8.0 * p.detail_weight * abs(lr - lb), 0.0, 1.0);
        outc = vec3<f32>(d, d, d);
    }
    textureStore(out_tex, vec2<i32>(x, y), vec4<f32>(outc, 1.0));
}
