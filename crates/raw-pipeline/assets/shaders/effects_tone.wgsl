struct EffectsToneParams {
    size: vec2<u32>,
    vignette: vec4<f32>,
    grain: vec3<f32>,
    display_p3: u32,
    warn_flags: u32,
    output_scale: f32,
    roi: vec4<f32>,
};

const GRAIN_SEED: u32 = 0x6A1A5EEDu;
const FINE_SEED: u32 = 0x6A1A5EEDu ^ 0x9E3779B9u;
const SPREAD_SALT: u32 = 0x85EBCA6Bu;
const EXACT_SPAN: i32 = 5;
const LSTAR_EPSILON: f32 = 216.0 / 24389.0;
const LSTAR_KAPPA: f32 = 24389.0 / 2700.0;
const GRAIN_LIGHTNESS: f32 = 0.19;

@group(0) @binding(0) var<uniform> p: EffectsToneParams;
@group(0) @binding(1) var src_lin: texture_2d<f32>;
// DISPLAY_STORE_INJECT
@group(0) @binding(3) var out_lin: texture_storage_2d<rgba16float, write>;

// TONE_WGSL_INJECT

// DISPLAY_CURVES_INJECT

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

// VIGNETTE_CONST_INJECT

fn vignette_gain(strength: f32, l: f32) -> f32 {
    if (strength >= 0.0) {
        return 1.0 + strength;
    }
    let highlight = smoothstep_f(VIGNETTE_HIGHLIGHT_LO, VIGNETTE_HIGHLIGHT_HI, l);
    let protect = 1.0 - VIGNETTE_HIGHLIGHT_PRIORITY * highlight;
    return exp2(strength * VIGNETTE_DARKEN_STOPS * protect);
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

fn tent(t: f32) -> f32 {
    let a = abs(t);
    return select(1.0 - fade(a), 0.0, a >= 1.0);
}

fn lightness(y: f32) -> f32 {
    return select(y * LSTAR_KAPPA, 1.16 * pow(y, 1.0 / 3.0) - 0.16, y > LSTAR_EPSILON);
}

fn luminance(l: f32) -> f32 {
    let c = (l + 0.16) / 1.16;
    return select(l / LSTAR_KAPPA, c * c * c, l > LSTAR_EPSILON * LSTAR_KAPPA);
}

fn lattice_lo(win: vec2<f32>, cell: f32) -> i32 {
    return i32(floor(floor(win.x) / cell));
}

fn lattice_count(win: vec2<f32>, cell: f32) -> i32 {
    let hi = i32(floor((ceil(win.y) - 1.0) / cell)) + 1;
    return hi - lattice_lo(win, cell) + 1;
}

fn lattice_weight(i: i32, win: vec2<f32>, cell: f32) -> f32 {
    let first = i32(max(floor(win.x), floor(f32(i - 1) * cell)));
    let last = i32(min(ceil(win.y) - 1.0, ceil(f32(i + 1) * cell)));
    var acc = 0.0;
    for (var xs = first; xs <= last; xs = xs + 1) {
        let cover = min(f32(xs + 1), win.y) - max(f32(xs), win.x);
        acc = acc + max(cover, 0.0) * tent(f32(xs) / cell - f32(i));
    }
    return acc / (win.y - win.x);
}

fn box_noise(wx: vec2<f32>, wy: vec2<f32>, cell: f32, seed: u32) -> f32 {
    let x0 = lattice_lo(wx, cell);
    let y0 = lattice_lo(wy, cell);
    let nx = lattice_count(wx, cell);
    let ny = lattice_count(wy, cell);
    if (nx <= EXACT_SPAN && ny <= EXACT_SPAN) {
        var ay: array<f32, 5>;
        for (var ky = 0; ky < ny; ky = ky + 1) {
            ay[ky] = lattice_weight(y0 + ky, wy, cell);
        }
        var sum = 0.0;
        for (var kx = 0; kx < nx; kx = kx + 1) {
            let ax = lattice_weight(x0 + kx, wx, cell);
            for (var ky = 0; ky < ny; ky = ky + 1) {
                sum = sum + hash2(x0 + kx, y0 + ky, seed) * ax * ay[ky];
            }
        }
        return sum;
    }
    var ex = 0.0;
    for (var kx = 0; kx < nx; kx = kx + 1) {
        let a = lattice_weight(x0 + kx, wx, cell);
        ex = ex + a * a;
    }
    var ey = 0.0;
    for (var ky = 0; ky < ny; ky = ky + 1) {
        let a = lattice_weight(y0 + ky, wy, cell);
        ey = ey + a * a;
    }
    let u = hash2(i32(floor(wx.x)), i32(floor(wy.x)), seed ^ SPREAD_SALT);
    return 0.5 + (u - 0.5) * sqrt(ex * ey);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let width = p.size.x;
    let height = p.size.y;
    if (gid.x >= width || gid.y >= height) { return; }
    let x = i32(gid.x);
    let y = i32(gid.y);

    var lin = textureLoad(src_lin, vec2<i32>(x, y), 0).rgb;

    let full_w = f32(width) / p.roi.z;
    let full_h = f32(height) / p.roi.w;

    let vig_amount = p.vignette.x;
    if (vig_amount != 0.0) {
        let midpoint = p.vignette.y;
        let feather = p.vignette.z;
        let roundness = (p.vignette.w + 1.0) * 0.5;
        let inner = mix(0.10, 0.90, midpoint);
        let band = mix(0.02, max(0.02, 1.25 - inner), feather);
        let aspect = full_w / full_h;
        let inv_w = 1.0 / f32(width);
        let inv_h = 1.0 / f32(height);
        let u_p = ((p.roi.x + (f32(x) + 0.5) * inv_w * p.roi.z) - 0.5) * 2.0;
        let v_p = ((p.roi.y + (f32(y) + 0.5) * inv_h * p.roi.w) - 0.5) * 2.0;
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
        let gain = vignette_gain(vig_amount * t, luma(lin));
        lin = clamp(lin * gain, vec3<f32>(0.0), vec3<f32>(4.0));
    }

    let grain_amount = p.grain.x;
    if (grain_amount != 0.0) {
        let size = p.grain.y;
        let roughness = p.grain.z;
        let cell = mix(1.0, 8.0, size);
        let fine_cell = max(1.0, cell * 0.5);
        let step = 1.0 / p.output_scale;
        let xf = p.roi.x * full_w + f32(x);
        let yf = p.roi.y * full_h + f32(y);
        let wx = vec2<f32>(xf, xf + 1.0) * step;
        let wy = vec2<f32>(yf, yf + 1.0) * step;
        let base = box_noise(wx, wy, cell, GRAIN_SEED);
        let fine = box_noise(wx, wy, fine_cell, FINE_SEED);
        let n = mix(base, fine, roughness) * 2.0 - 1.0;
        let yv = luma(lin);
        if (yv > 0.0) {
            let l = lightness(yv);
            let midtone = max(4.0 * l * (1.0 - l), 0.0);
            let scale = luminance(l + n * grain_amount * GRAIN_LIGHTNESS * midtone) / yv;
            lin = clamp(lin * scale, vec3<f32>(0.0), vec3<f32>(4.0));
        }
    }

    textureStore(out_lin, vec2<i32>(x, y), vec4<f32>(lin, 1.0));
    let outc = display_curves_apply(tone_rgb_cs(lin, p.display_p3));
    let outc_d = tone_dither_u8(outc, gid.x, gid.y);
    var alpha = 1.0;
    if ((p.warn_flags & 1u) != 0u && tone_is_out_of_gamut(lin, p.display_p3)) {
        alpha = 0.0;
    } else if ((p.warn_flags & 2u) != 0u) {
        alpha = warn_clip_alpha(outc);
    }
    store_display(vec2<i32>(x, y), vec4<f32>(outc_d, alpha));
}
