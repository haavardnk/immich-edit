fn color_grade_apply(c: vec3<f32>) -> vec3<f32> {
    let balance = p.color_grade[4].x;
    let blend = p.color_grade[4].y;
    let lab = op_to_oklab(c);
    let y = clamp(lab.x, 0.0, 1.0);
    let pivot = COLOR_GRADE_PIVOT_BASE + COLOR_GRADE_PIVOT_RANGE * balance;
    let feather = COLOR_GRADE_FEATHER_BASE + COLOR_GRADE_FEATHER_RANGE * blend;
    let s_hi = clamp(pivot + feather * 0.5, 0.001, 0.999);
    let s_lo = clamp(pivot - feather - feather * 0.5, 0.0, s_hi - 0.001);
    let h_lo = clamp(pivot - feather * 0.5, 0.001, 0.999);
    let h_hi = clamp(pivot + feather + feather * 0.5, h_lo + 0.001, 1.0);
    let ws = 1.0 - smoothstep(s_lo, s_hi, y);
    let wh = smoothstep(h_lo, h_hi, y);
    let wm = max(1.0 - ws - wh, 0.0);
    let off = p.color_grade[0].xyz * ws + p.color_grade[1].xyz * wm + p.color_grade[2].xyz * wh + p.color_grade[3].xyz;
    let graded = op_from_oklab(vec3<f32>(lab.x, lab.yz + max(lab.x, 0.0) * off.xy));
    return graded * exp2(off.z);
}
