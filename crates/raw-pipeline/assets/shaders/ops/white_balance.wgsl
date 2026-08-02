fn white_balance_apply(c: vec3<f32>, w: vec4<f32>) -> vec3<f32> {{
    var wb_lin = vec3<f32>(c.r * w.r, c.g * w.g, c.b * w.b);
    if (w.w < 0.5) {{ return wb_lin; }}
    let cr = smoothstep({knee:?}, {white:?}, c.r);
    let cg = smoothstep({knee:?}, {white:?}, c.g);
    let cb = smoothstep({knee:?}, {white:?}, c.b);
    if (max(cr, max(cg, cb)) <= 0.0) {{ return wb_lin; }}
    let ur = 1.0 - cr;
    let ug = 1.0 - cg;
    let ub = 1.0 - cb;
    let wmax = max(wb_lin.r, max(wb_lin.g, wb_lin.b));
    let recon_target = (ur * wb_lin.r + ug * wb_lin.g + ub * wb_lin.b + {bias:?} * wmax) / (ur + ug + ub + {bias:?});
    if (wb_lin.r < recon_target) {{ wb_lin.r = wb_lin.r + (recon_target - wb_lin.r) * cr; }}
    if (wb_lin.g < recon_target) {{ wb_lin.g = wb_lin.g + (recon_target - wb_lin.g) * cg; }}
    if (wb_lin.b < recon_target) {{ wb_lin.b = wb_lin.b + (recon_target - wb_lin.b) * cb; }}
    return wb_lin;
}}
