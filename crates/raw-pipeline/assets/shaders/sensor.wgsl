struct SensorParams {
    size: vec2<u32>,
    zoom: f32,
    vig_amount: f32,
    coeffs: vec4<f32>,
    ca_vig: vec4<f32>,
};

@group(0) @binding(0) var<uniform> p: SensorParams;
@group(0) @binding(1) var src: texture_2d<f32>;
@group(0) @binding(2) var dst: texture_storage_2d<rgba16float, write>;

fn load_clamped(ix: i32, iy: i32) -> vec3<f32> {
    let w = i32(p.size.x);
    let h = i32(p.size.y);
    let cx = clamp(ix, 0, w - 1);
    let cy = clamp(iy, 0, h - 1);
    return textureLoad(src, vec2<i32>(cx, cy), 0).rgb;
}

fn bilinear_rgb(x: f32, y: f32) -> vec3<f32> {
    let ix = i32(floor(x));
    let iy = i32(floor(y));
    let tx = x - floor(x);
    let ty = y - floor(y);
    let c00 = load_clamped(ix, iy);
    let c10 = load_clamped(ix + 1, iy);
    let c01 = load_clamped(ix, iy + 1);
    let c11 = load_clamped(ix + 1, iy + 1);
    let a = mix(c00, c10, tx);
    let b = mix(c01, c11, tx);
    return mix(a, b, ty);
}

fn bilinear_chan(x: f32, y: f32, ch: i32) -> f32 {
    let s = bilinear_rgb(x, y);
    if (ch == 0) { return s.r; }
    if (ch == 1) { return s.g; }
    return s.b;
}

fn distortion_scale(r2: f32) -> f32 {
    let k1 = p.coeffs.x;
    let k2 = p.coeffs.y;
    let k3 = p.coeffs.z;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    return 1.0 + k1 * r2 + k2 * r4 + k3 * r6;
}

fn vignette_correction(r_norm: f32) -> f32 {
    let vk1 = p.coeffs.w;
    let vk2 = p.ca_vig.z;
    let vk3 = p.ca_vig.w;
    let amount = p.vig_amount;
    let r2 = r_norm * r_norm;
    let r4 = r2 * r2;
    let r6 = r4 * r2;
    let poly = 1.0 + vk1 * r2 + vk2 * r4 + vk3 * r6;
    var full_gain = 1.0;
    if (abs(poly) >= 1e-6) { full_gain = 1.0 / poly; }
    let gain = 1.0 + (full_gain - 1.0) * amount;
    return clamp(gain, 0.25, 2.5);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let w = f32(p.size.x);
    let h = f32(p.size.y);
    let cx = w * 0.5;
    let cy = h * 0.5;
    let half_diag = 0.5 * sqrt(w * w + h * h);
    let inv_norm = 1.0 / half_diag;

    let dx = (f32(gid.x) + 0.5 - cx) * p.zoom;
    let dy = (f32(gid.y) + 0.5 - cy) * p.zoom;

    let ca_red = p.ca_vig.x;
    let ca_blue = p.ca_vig.y;

    let dxr = dx * ca_red;
    let dyr = dy * ca_red;
    let rr = sqrt(dxr * dxr + dyr * dyr) * inv_norm;
    let sr = distortion_scale(rr * rr);
    let srx = dxr * sr + cx - 0.5;
    let sry = dyr * sr + cy - 0.5;
    let r_red = bilinear_chan(srx, sry, 0) * vignette_correction(rr);

    let rg = sqrt(dx * dx + dy * dy) * inv_norm;
    let sg = distortion_scale(rg * rg);
    let sgx = dx * sg + cx - 0.5;
    let sgy = dy * sg + cy - 0.5;
    let g_green = bilinear_chan(sgx, sgy, 1) * vignette_correction(rg);

    let dxb = dx * ca_blue;
    let dyb = dy * ca_blue;
    let rb = sqrt(dxb * dxb + dyb * dyb) * inv_norm;
    let sb = distortion_scale(rb * rb);
    let sbx = dxb * sb + cx - 0.5;
    let sby = dyb * sb + cy - 0.5;
    let b_blue = bilinear_chan(sbx, sby, 2) * vignette_correction(rb);

    textureStore(dst, vec2<i32>(i32(gid.x), i32(gid.y)), vec4<f32>(max(r_red, 0.0), max(g_green, 0.0), max(b_blue, 0.0), 1.0));
}
