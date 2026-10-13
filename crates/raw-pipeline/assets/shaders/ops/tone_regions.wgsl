fn tone_regions_whites_mult(luma: f32, wh: f32) -> f32 {
    if (wh == 0.0) { return 1.0; }
    let l = max(luma, 0.0);
    return exp2(wh * TONE_REGIONS_WHITES_STOPS * l / (l + TONE_REGIONS_WHITES_PIVOT));
}

fn tone_regions_highlights_gain(peak: f32, hl: f32) -> f32 {
    let w = smoothstep(TONE_REGIONS_HL_COMPRESS_LO, TONE_REGIONS_HL_COMPRESS_HI, log2(max(peak, 1e-6)));
    return exp2(hl * TONE_REGIONS_HL_STRENGTH * w);
}

fn tone_regions_highlights_lift(x: f32, hl: f32) -> f32 {
    let mask = smoothstep(TONE_REGIONS_HL_MASK_LO, TONE_REGIONS_HL_MASK_HI, tanh(x * TONE_REGIONS_HL_MASK_TANH));
    return x + x * (exp2(hl * TONE_REGIONS_HL_STRENGTH) - 1.0) * mask;
}

fn tone_regions_shadows_blur_at(blur_l: f32, source_l: f32, luma: f32) -> f32 {
    return blur_l * luma / max(source_l, TONE_REGIONS_SH_SOURCE_FLOOR);
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
    var mask_bk = clamp(1.0 - max(x, 0.0) / TONE_REGIONS_BK_MASK_RANGE, 0.0, 1.0);
    mask_bk = mask_bk * mask_bk;
    let mult_bk = clamp(exp2(bk * TONE_REGIONS_BK_STRENGTH), 0.0, TONE_REGIONS_BK_MULT_MAX);
    return x + x * (mult_bk - 1.0) * mask_bk;
}

fn tone_regions_apply_rgb(c: vec3<f32>, hl: f32, bk: f32) -> vec3<f32> {
    var v = c;
    if (hl < 0.0) {
        v = c * tone_regions_highlights_gain(max(max(c.x, c.y), c.z), hl);
    } else if (hl > 0.0) {
        let lo = min(min(c.x, c.y), c.z);
        let hi = max(max(c.x, c.y), c.z);
        v = op_rgb_tone(c, tone_regions_highlights_lift(lo, hl), tone_regions_highlights_lift(hi, hl));
    }
    if (bk == 0.0) { return v; }
    let lo = min(min(v.x, v.y), v.z);
    let hi = max(max(v.x, v.y), v.z);
    return op_rgb_tone(v, tone_regions_blacks(lo, bk), tone_regions_blacks(hi, bk));
}

fn tone_regions_apply(c: vec3<f32>, p: vec4<f32>, blur_l: f32, source_l: f32) -> vec3<f32> {
    if (p.x == 0.0 && p.y == 0.0 && p.z == 0.0 && p.w == 0.0) { return c; }
    let src_luma = 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
    var v = c * tone_regions_whites_mult(src_luma, p.w);
    let luma = 0.2126 * v.x + 0.7152 * v.y + 0.0722 * v.z;
    let sm = tone_regions_shadows_mult(luma, tone_regions_shadows_blur_at(blur_l, source_l, luma), p.y);
    v = v * sm;
    return tone_regions_apply_rgb(v, p.x, p.z);
}
