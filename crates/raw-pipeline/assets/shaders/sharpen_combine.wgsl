struct CombineParams {
    sharpen: vec4<f32>,
    dims_flags: vec4<u32>,
    vignette: vec4<f32>,
    grain: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: CombineParams;
@group(0) @binding(1) var src_lin: texture_2d<f32>;
@group(0) @binding(2) var src_blur: texture_2d<f32>;
@group(0) @binding(3) var out_tex: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(4) var out_lin: texture_storage_2d<rgba16float, write>;

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

fn fade(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

fn pcg_hash(seed: u32) -> u32 {
    var x = seed * 747796405u + 2891336453u;
    let word = ((x >> ((x >> 28u) + 4u)) ^ x) * 277803737u;
    return (word >> 22u) ^ word;
}

fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let h = pcg_hash((u32(x) * 0x27d4eb2du) ^ pcg_hash(u32(y) ^ seed));
    return f32(h) / 4294967295.0;
}

fn value_noise(x: f32, y: f32, seed: u32) -> f32 {
    let xi = i32(floor(x));
    let yi = i32(floor(y));
    let xf = x - floor(x);
    let yf = y - floor(y);
    let u = fade(xf);
    let v = fade(yf);
    let a = hash2(xi, yi, seed);
    let b = hash2(xi + 1, yi, seed);
    let c = hash2(xi, yi + 1, seed);
    let d = hash2(xi + 1, yi + 1, seed);
    return mix(mix(a, b, u), mix(c, d, u), v);
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
    var lin = orig + hp * strength;

    if (preview_mode == 0u) {
        let vig_amount = p.vignette.x;
        if (vig_amount != 0.0) {
            let midpoint = p.vignette.y;
            let feather = p.vignette.z;
            let roundness = (p.vignette.w + 1.0) * 0.5;
            let inner = mix(0.10, 0.90, midpoint);
            let band = mix(0.02, max(0.02, 1.25 - inner), feather);
            let aspect = f32(width) / f32(height);
            let inv_w = 1.0 / f32(width);
            let inv_h = 1.0 / f32(height);
            let u_p = ((f32(x) + 0.5) * inv_w - 0.5) * 2.0;
            let v_p = ((f32(y) + 0.5) * inv_h - 0.5) * 2.0;
            var cx: f32;
            var cy: f32;
            if (aspect >= 1.0) {
                cx = u_p * aspect;
                cy = v_p;
            } else {
                cx = u_p;
                cy = v_p / aspect;
            }
            let qx = mix(u_p, cx, roundness);
            let qy = mix(v_p, cy, roundness);
            let d = sqrt(qx * qx + qy * qy);
            let t = smoothstep_f(inner, inner + band, d);
            let gain = clamp(1.0 + vig_amount * t, 0.0, 2.0);
            lin = clamp(lin * gain, vec3<f32>(0.0), vec3<f32>(4.0));
        }
        let grain_amount = p.grain.x;
        if (grain_amount != 0.0) {
            let size = p.grain.y;
            let roughness = p.grain.z;
            let cell = mix(1.0, 8.0, size);
            let fine_cell = max(1.0, cell * 0.5);
            let seed = width ^ ((height << 13u) | (height >> 19u));
            let seed_fine = seed ^ 0x9E3779B9u;
            let base = value_noise(f32(x) / cell, f32(y) / cell, seed);
            let fine = value_noise(f32(x) / fine_cell, f32(y) / fine_cell, seed_fine);
            let n = mix(base, fine, roughness) * 2.0 - 1.0;
            let delta = n * grain_amount * 0.15;
            let yv = luma(lin);
            let scale = select(1.0, (yv + delta) / yv, yv > 1e-6);
            lin = clamp(lin * scale, vec3<f32>(0.0), vec3<f32>(4.0));
        }
    }

    textureStore(out_lin, vec2<i32>(x, y), vec4<f32>(lin, 1.0));

    var outc = vec3<f32>(default_tone(lin.r), default_tone(lin.g), default_tone(lin.b));
    if (preview_mode == 1u) {
        outc = vec3<f32>(mask, mask, mask);
    } else if (preview_mode == 2u) {
        let lb = luma(blur);
        outc = vec3<f32>(lb, lb, lb);
    } else if (preview_mode == 3u) {
        let lr = luma(orig);
        let lb = luma(blur);
        let d = clamp(8.0 * detail_weight * abs(lr - lb), 0.0, 1.0);
        outc = vec3<f32>(d, d, d);
    }
    textureStore(out_tex, vec2<i32>(x, y), vec4<f32>(outc, 1.0));
}
