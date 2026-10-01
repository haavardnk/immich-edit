struct Taps {
    idx: vec2<i32>,
    w: vec2<f32>,
}

fn nr_luma(c: vec3<f32>) -> f32 {
    return LUMA_R * c.r + LUMA_G * c.g + LUMA_B * c.b;
}

fn to_chroma(c: vec3<f32>) -> vec3<f32> {
    let y = nr_luma(c);
    return vec3<f32>(y, (c.b - y) / PB_DEN, (c.r - y) / PR_DEN);
}

fn from_chroma(y: f32, pb: f32, pr: f32) -> vec3<f32> {
    let r = y + PR_DEN * pr;
    let b = y + PB_DEN * pb;
    return vec3<f32>(r, (y - LUMA_R * r - LUMA_B * b) / LUMA_G, b);
}

fn hist_index(d: f32, reference: f32) -> i32 {
    let m = abs(d);
    if (!(m > 0.0)) {
        return -1;
    }
    let lum = min(u32(sqrt(clamp(reference, 0.0, 1.0)) * f32(LUM_BINS)), LUM_BINS - 1u);
    let mag = clamp(floor((log2(m) - MAG_LOG2_MIN) * MAG_BINS_PER_OCTAVE), 0.0, f32(MAG_BINS - 1u));
    return i32(lum * MAG_BINS + u32(mag));
}

fn noise_variance(a: f32, b: f32, y: f32) -> f32 {
    return max(a + b * max(y, 0.0), 0.0);
}

fn local_energy(total: f32, centre: f32) -> f32 {
    let around = total - centre;
    return (around + min(centre, around / 8.0)) / 9.0;
}

fn shrink_gain(energy: f32, variance: f32, lambda: f32, mu: f32, keep: f32) -> f32 {
    let noise = variance * lambda;
    if (noise <= 0.0) {
        return 1.0;
    }
    let signal = max(energy - mu * noise, 0.0);
    let g = signal / (signal + noise);
    return g + (1.0 - g) * keep;
}

fn up_taps(x: u32, n: u32) -> Taps {
    let half = i32(x / 2u);
    let last = i32(n) - 1;
    if ((x & 1u) == 0u) {
        return Taps(vec2<i32>(max(half - 1, 0), min(half, last)), vec2<f32>(0.25, 0.75));
    }
    return Taps(vec2<i32>(min(half, last), min(half + 1, last)), vec2<f32>(0.75, 0.25));
}

fn upsample(t: texture_2d<f32>, p: vec2<u32>, lo: vec2<u32>) -> vec4<f32> {
    let tx = up_taps(p.x, lo.x);
    let ty = up_taps(p.y, lo.y);
    let top = tx.w.x * textureLoad(t, vec2<i32>(tx.idx.x, ty.idx.x), 0)
        + tx.w.y * textureLoad(t, vec2<i32>(tx.idx.y, ty.idx.x), 0);
    let bottom = tx.w.x * textureLoad(t, vec2<i32>(tx.idx.x, ty.idx.y), 0)
        + tx.w.y * textureLoad(t, vec2<i32>(tx.idx.y, ty.idx.y), 0);
    return ty.w.x * top + ty.w.y * bottom;
}
