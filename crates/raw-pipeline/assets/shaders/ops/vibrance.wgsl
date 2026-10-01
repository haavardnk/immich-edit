fn vibrance_apply(c: vec3<f32>, amount: f32) -> vec3<f32> {
    if (amount == 0.0) { return c; }
    let lab = op_to_oklab(c);
    let chroma = length(lab.yz);
    if (lab.x <= 1e-6 || chroma < 1e-6) { return c; }
    let sat = chroma / lab.x;
    var effective: f32;
    if (amount > 0.0) {
        let base = amount * VIBRANCE_GAIN * (1.0 - smoothstep(VIBRANCE_SAT_LO, VIBRANCE_SAT_HI, sat));
        var skin = 1.0 - smoothstep(VIBRANCE_SKIN_SPREAD_LO_DEG, VIBRANCE_SKIN_SPREAD_HI_DEG, op_hue_dist(degrees(atan2(lab.z, lab.y)), VIBRANCE_SKIN_HUE_DEG));
        skin = skin * smoothstep(VIBRANCE_SKIN_SAT_LO, VIBRANCE_SKIN_SAT_HI, sat);
        effective = base * (1.0 + (VIBRANCE_SKIN_FACTOR - 1.0) * skin);
    } else {
        effective = amount * (1.0 - smoothstep(VIBRANCE_DESAT_LO, VIBRANCE_DESAT_HI, sat));
    }
    if (abs(effective) < 1e-5) { return c; }
    return op_from_oklab(vec3<f32>(lab.x, lab.yz * (1.0 + effective)));
}
