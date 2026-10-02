struct SelectorParams {
    size: vec2<u32>,
    src_size: vec2<u32>,
    offset: u32,
    mode: u32,
    whole: u32,
    frac: f32,
    eps: f32,
};

@group(0) @binding(0) var<uniform> p: SelectorParams;
@group(0) @binding(1) var in_a: texture_2d<f32>;
@group(0) @binding(2) var in_b: texture_2d<f32>;
@group(0) @binding(3) var guide: texture_2d<f32>;
@group(0) @binding(4) var out_a: texture_storage_2d<rgba16float, write>;
@group(0) @binding(5) var out_b: texture_storage_2d<rgba16float, write>;
@group(0) @binding(6) var out_full: texture_storage_2d<rgba32float, write>;

// TONE_WGSL_INJECT

fn tap_coord(base: vec2<i32>, k: i32) -> vec2<i32> {
    if (p.mode == 1u || p.mode == 3u) {
        return vec2<i32>(clamp(base.x + k, 0, i32(p.src_size.x) - 1), base.y);
    }
    return vec2<i32>(base.x, clamp(base.y + k, 0, i32(p.src_size.y) - 1));
}

fn tap_weight(k: i32) -> f32 {
    if (abs(k) <= i32(p.whole)) { return 1.0; }
    return p.frac;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let dst = vec2<i32>(i32(gid.x), i32(gid.y));
    if (p.mode == 0u) {
        let display = tone_apply_rgb(textureLoad(guide, dst, 0).rgb);
        textureStore(out_full, dst, vec4<f32>(display, 1.0));
        return;
    }
    let base = dst + vec2<i32>(i32(p.offset));
    let center = textureLoad(guide, base, 0).rgb;
    let reach = i32(p.whole) + 1;
    let norm = 2.0 * (f32(p.whole) + p.frac) + 1.0;
    var acc_a = vec3<f32>(0.0);
    var acc_b = vec3<f32>(0.0);
    var acc_c = vec3<f32>(0.0);
    for (var k: i32 = -reach; k <= reach; k = k + 1) {
        let w = tap_weight(k);
        let c = tap_coord(base, k);
        let g = textureLoad(guide, c, 0).rgb - center;
        switch p.mode {
            case 1u: {
                acc_a = acc_a + w * g;
                acc_b = acc_b + w * g * g;
            }
            case 2u: {
                let v = textureLoad(in_a, c, 0).rgb + g;
                acc_a = acc_a + w * v;
                acc_b = acc_b + w * v * v;
                acc_c = acc_c + w * textureLoad(in_b, c, 0).rgb;
            }
            case 3u: {
                let hold = textureLoad(in_a, c, 0).rgb;
                acc_a = acc_a + w * (hold * g + textureLoad(in_b, c, 0).rgb);
                acc_b = acc_b + w * hold;
            }
            default: {
                acc_a = acc_a + w * (textureLoad(in_a, c, 0).rgb + textureLoad(in_b, c, 0).rgb * g);
            }
        }
    }
    let mean_a = acc_a / norm;
    if (p.mode == 1u) {
        let spread = max(acc_b / norm - mean_a * mean_a, vec3<f32>(0.0));
        textureStore(out_a, dst, vec4<f32>(mean_a, 1.0));
        textureStore(out_b, dst, vec4<f32>(spread, 1.0));
        return;
    }
    if (p.mode == 2u) {
        let variance = max(acc_c / norm + acc_b / norm - mean_a * mean_a, vec3<f32>(0.0));
        let hold = vec3<f32>(p.eps) / (variance + vec3<f32>(p.eps));
        textureStore(out_a, dst, vec4<f32>(hold, 1.0));
        textureStore(out_b, dst, vec4<f32>(hold * mean_a, 1.0));
        return;
    }
    if (p.mode == 3u) {
        textureStore(out_a, dst, vec4<f32>(mean_a, 1.0));
        textureStore(out_b, dst, vec4<f32>(acc_b / norm, 1.0));
        return;
    }
    textureStore(out_full, dst, vec4<f32>(center + mean_a, 1.0));
}
