struct DemosaicParams {
    size: vec2<u32>,
    cfa: vec4<u32>,
};

@group(0) @binding(0) var<uniform> p: DemosaicParams;
@group(0) @binding(1) var<storage, read> raw_in: array<f32>;
@group(0) @binding(2) var rgb_out: texture_storage_2d<rgba16float, write>;

fn cfa_at(x: u32, y: u32) -> u32 {
    return p.cfa[(y & 1u) * 2u + (x & 1u)];
}

fn fetch(ix: i32, iy: i32) -> f32 {
    let xc = clamp(ix, 0, i32(p.size.x) - 1);
    let yc = clamp(iy, 0, i32(p.size.y) - 1);
    let idx = u32(yc) * p.size.x + u32(xc);
    return raw_in[idx];
}

fn avg_color(ix: i32, iy: i32, want: u32) -> f32 {
    var sum: f32 = 0.0;
    var n: f32 = 0.0;
    for (var dy = -1; dy <= 1; dy = dy + 1) {
        for (var dx = -1; dx <= 1; dx = dx + 1) {
            let x = ix + dx;
            let y = iy + dy;
            let xc = clamp(x, 0, i32(p.size.x) - 1);
            let yc = clamp(y, 0, i32(p.size.y) - 1);
            if (cfa_at(u32(xc), u32(yc)) == want) {
                sum = sum + fetch(xc, yc);
                n = n + 1.0;
            }
        }
    }
    if (n == 0.0) { return 0.0; }
    return sum / n;
}

fn bilinear_rgb(ix: i32, iy: i32) -> vec3<f32> {
    let own_c = cfa_at(u32(ix), u32(iy));
    let own_v = fetch(ix, iy);
    var r: f32;
    var g: f32;
    var b: f32;
    if (own_c == 0u) {
        r = own_v;
        g = avg_color(ix, iy, 1u);
        b = avg_color(ix, iy, 2u);
    } else if (own_c == 2u) {
        r = avg_color(ix, iy, 0u);
        g = avg_color(ix, iy, 1u);
        b = own_v;
    } else {
        r = avg_color(ix, iy, 0u);
        g = own_v;
        b = avg_color(ix, iy, 2u);
    }
    return vec3<f32>(r, g, b);
}

fn fetch_unchecked(ix: i32, iy: i32) -> f32 {
    let idx = u32(iy) * p.size.x + u32(ix);
    return raw_in[idx];
}

fn mhc_rgb(ix: i32, iy: i32) -> vec3<f32> {
    let own_c = cfa_at(u32(ix), u32(iy));
    let c = fetch_unchecked(ix, iy);

    let pm10 = fetch_unchecked(ix - 1, iy);
    let pp10 = fetch_unchecked(ix + 1, iy);
    let p0m1 = fetch_unchecked(ix, iy - 1);
    let p0p1 = fetch_unchecked(ix, iy + 1);
    let pm20 = fetch_unchecked(ix - 2, iy);
    let pp20 = fetch_unchecked(ix + 2, iy);
    let p0m2 = fetch_unchecked(ix, iy - 2);
    let p0p2 = fetch_unchecked(ix, iy + 2);
    let pmm = fetch_unchecked(ix - 1, iy - 1);
    let ppm = fetch_unchecked(ix + 1, iy - 1);
    let pmp = fetch_unchecked(ix - 1, iy + 1);
    let ppp = fetch_unchecked(ix + 1, iy + 1);

    var r: f32 = 0.0;
    var g: f32 = 0.0;
    var b: f32 = 0.0;

    if (own_c == 1u) {
        let row_ch = cfa_at(u32(ix + 1), u32(iy));
        let col_ch = cfa_at(u32(ix), u32(iy + 1));
        let n1 = pm10 + pp10;
        let n2 = p0m1 + p0p1;
        let d2 = pm20 + pp20;
        let d2v = p0m2 + p0p2;
        let diag = pmm + ppm + pmp + ppp;
        let h_val = clamp((n1 * 4.0 + c * 5.0 - d2 - diag + d2v * 0.5) / 8.0, 0.0, 1.0);
        let v_val = clamp((n2 * 4.0 + c * 5.0 - d2v - diag + d2 * 0.5) / 8.0, 0.0, 1.0);
        if (row_ch == 0u) { r = h_val; } else if (row_ch == 2u) { b = h_val; }
        if (col_ch == 0u) { r = v_val; } else if (col_ch == 2u) { b = v_val; }
        g = c;
    } else {
        let n4 = pm10 + pp10 + p0m1 + p0p1;
        let dplus = pm20 + pp20 + p0m2 + p0p2;
        let g_val = clamp((n4 * 2.0 + c * 4.0 - dplus) / 8.0, 0.0, 1.0);
        let diag = pmm + ppm + pmp + ppp;
        let opp_val = clamp((diag * 2.0 + c * 6.0 - dplus * 1.5) / 8.0, 0.0, 1.0);
        g = g_val;
        if (own_c == 0u) {
            r = c;
            b = opp_val;
        } else {
            r = opp_val;
            b = c;
        }
    }
    return vec3<f32>(r, g, b);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    if (gid.x >= p.size.x || gid.y >= p.size.y) { return; }
    let ix = i32(gid.x);
    let iy = i32(gid.y);

    var rgb: vec3<f32>;
    if (p.size.x < 5u || p.size.y < 5u
        || gid.x < 2u || gid.y < 2u
        || gid.x >= p.size.x - 2u || gid.y >= p.size.y - 2u) {
        rgb = bilinear_rgb(ix, iy);
    } else {
        rgb = mhc_rgb(ix, iy);
    }
    textureStore(rgb_out, vec2<i32>(ix, iy), vec4<f32>(rgb, 1.0));
}
