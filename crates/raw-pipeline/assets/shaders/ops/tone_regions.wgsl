fn tone_regions_whites_gain(wh: f32) -> f32 {
    return 1.0 / (1.0 - clamp(wh, -TONE_REGIONS_WHITES_CLAMP, TONE_REGIONS_WHITES_CLAMP) * TONE_REGIONS_WHITES_SCALE);
}

fn tone_regions_highlights(x: f32, hl: f32) -> f32 {
    if (hl == 0.0) { return x; }
    let mask = smoothstep(TONE_REGIONS_HL_MASK_LO, TONE_REGIONS_HL_MASK_HI, tanh(x * TONE_REGIONS_HL_MASK_TANH));
    var new_v: f32;
    if (hl < 0.0) {
        let gamma = 1.0 - hl * TONE_REGIONS_HL_STRENGTH;
        let base = pow(max(min(x, 1.0), 0.0), gamma);
        let excess = max(x - 1.0, 0.0);
        let blend = max(1.0 + hl, 0.0);
        new_v = base + excess * blend;
    } else {
        new_v = x * exp2(hl * TONE_REGIONS_HL_STRENGTH);
    }
    return x * (1.0 - mask) + new_v * mask;
}

fn tone_regions_shadows_mult(luma: f32, blur_l: f32, sh: f32) -> f32 {
    if (sh == 0.0) { return 1.0; }
    var mask = clamp(1.0 - luma / TONE_REGIONS_SH_MASK_RANGE, 0.0, 1.0);
    mask = mask * mask;
    let edge = abs(sqrt(max(luma, 0.0)) - sqrt(max(blur_l, 0.0)));
    let halo = 1.0 - smoothstep(TONE_REGIONS_SH_HALO_LO, TONE_REGIONS_SH_HALO_HI, edge);
    let mult = clamp(exp2(sh * TONE_REGIONS_SH_STRENGTH * halo), TONE_REGIONS_SH_MULT_MIN, TONE_REGIONS_SH_MULT_MAX);
    return 1.0 + (mult - 1.0) * mask;
}

fn tone_regions_blacks(x: f32, bk: f32) -> f32 {
    let xc = clamp(x, 0.0, TONE_REGIONS_BK_CEILING);
    var mask_bk = clamp(1.0 - xc / TONE_REGIONS_BK_MASK_RANGE, 0.0, 1.0);
    mask_bk = mask_bk * mask_bk;
    let mult_bk = clamp(exp2(bk * TONE_REGIONS_BK_STRENGTH), 0.0, TONE_REGIONS_BK_MULT_MAX);
    return xc + xc * (mult_bk - 1.0) * mask_bk;
}

fn tone_regions_apply_rgb(c: vec3<f32>, hl: f32, bk: f32) -> vec3<f32> {
    let clip = max(max(max(c.r, c.g), c.b) - 1.0, 0.0);
    var v = vec3<f32>(
        tone_regions_highlights(c.r, hl),
        tone_regions_highlights(c.g, hl),
        tone_regions_highlights(c.b, hl)
    );
    let desat = smoothstep(TONE_REGIONS_HL_DESAT_LO, TONE_REGIONS_HL_DESAT_HI, clip) * clamp(-hl, 0.0, 1.0);
    if (desat > 0.0) {
        let luma = 0.2126 * v.x + 0.7152 * v.y + 0.0722 * v.z;
        v = mix(v, vec3<f32>(luma), desat);
    }
    return vec3<f32>(
        tone_regions_blacks(v.x, bk),
        tone_regions_blacks(v.y, bk),
        tone_regions_blacks(v.z, bk)
    );
}

fn tone_regions_apply(c: vec3<f32>, p: vec4<f32>, blur_l: f32) -> vec3<f32> {
    if (p.x == 0.0 && p.y == 0.0 && p.z == 0.0 && p.w == 0.0) { return c; }
    let gain = tone_regions_whites_gain(p.w);
    var v = c * gain;
    let luma = 0.2126 * v.x + 0.7152 * v.y + 0.0722 * v.z;
    let sm = tone_regions_shadows_mult(luma, blur_l, p.y);
    v = v * sm;
    return tone_regions_apply_rgb(v, p.x, p.z);
}
