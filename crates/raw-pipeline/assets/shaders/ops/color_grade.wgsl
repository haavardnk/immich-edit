fn cg_hue_dir(hue_deg: f32) -> vec3<f32> {
    let h = (hue_deg - floor(hue_deg / 360.0) * 360.0) / 60.0;
    let x = 1.0 - abs((h - floor(h / 2.0) * 2.0) - 1.0);
    let i = i32(floor(h));
    var rgb: vec3<f32>;
    if (i == 0) { rgb = vec3<f32>(1.0, x, 0.0); }
    else if (i == 1) { rgb = vec3<f32>(x, 1.0, 0.0); }
    else if (i == 2) { rgb = vec3<f32>(0.0, 1.0, x); }
    else if (i == 3) { rgb = vec3<f32>(0.0, x, 1.0); }
    else if (i == 4) { rgb = vec3<f32>(x, 0.0, 1.0); }
    else { rgb = vec3<f32>(1.0, 0.0, x); }
    return rgb - vec3<f32>(0.5);
}

fn cg_region_offset(reg: vec4<f32>) -> vec4<f32> {
    let dir = cg_hue_dir(reg.x) * reg.y;
    return vec4<f32>(dir.x, dir.y, dir.z, reg.z);
}

fn color_grade_apply(c: vec3<f32>) -> vec3<f32> {
    let s = cg_region_offset(p.color_grade[0]);
    let m = cg_region_offset(p.color_grade[1]);
    let h = cg_region_offset(p.color_grade[2]);
    let g = cg_region_offset(p.color_grade[3]);
    let balance = p.color_grade[4].x;
    let blend = p.color_grade[4].y;
    let y = clamp(0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b, 0.0, 1.0);
    let pivot = 0.5 + 0.3 * balance;
    let feather = 0.15 + 0.25 * blend;
    let s_hi = clamp(pivot + feather * 0.5, 0.001, 0.999);
    let s_lo = clamp(pivot - feather - feather * 0.5, 0.0, s_hi - 0.001);
    let h_lo = clamp(pivot - feather * 0.5, 0.001, 0.999);
    let h_hi = clamp(pivot + feather + feather * 0.5, h_lo + 0.001, 1.0);
    let ws = 1.0 - smoothstep(s_lo, s_hi, y);
    let wh = smoothstep(h_lo, h_hi, y);
    let wm = max(1.0 - ws - wh, 0.0);
    let strength = 0.5;
    let off = (vec3<f32>(s.x, s.y, s.z) * ws + vec3<f32>(m.x, m.y, m.z) * wm + vec3<f32>(h.x, h.y, h.z) * wh + vec3<f32>(g.x, g.y, g.z)) * strength;
    let lum = (s.w * ws + m.w * wm + h.w * wh + g.w) * strength;
    return max(c + off + vec3<f32>(lum), vec3<f32>(0.0));
}
