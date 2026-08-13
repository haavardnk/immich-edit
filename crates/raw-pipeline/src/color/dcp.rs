use super::hsv::{hsv_to_rgb, rgb_to_hsv, srgb_degamma, srgb_gamma};
use super::matrix::{
    D50_XY, bradford_cat, cam_to_srgb_matrix, estimate_scene_cct, mat3_mul, mat3_vec,
};
use super::{D65_XY, XYZ_TO_SRGB_D65};
use crate::dcp::{DcpIlluminant, DcpProfile, HsvEncoding, HueSatMap};

pub fn dcp_illuminant_cct(code: u16) -> f32 {
    match code {
        1 => 5500.0,
        2 => 4150.0,
        3 => 2856.0,
        4 => 5500.0,
        9 => 5500.0,
        10 => 6500.0,
        11 => 7500.0,
        12 => 6430.0,
        13 => 5000.0,
        14 => 4150.0,
        15 => 3450.0,
        17 => 2856.0,
        18 => 4874.0,
        19 => 6774.0,
        20 => 5503.0,
        21 => 6504.0,
        22 => 7504.0,
        23 => 5003.0,
        24 => 3200.0,
        _ => 6504.0,
    }
}

fn pad4(m: [[f32; 3]; 3]) -> [[f32; 3]; 4] {
    [m[0], m[1], m[2], [0.0; 3]]
}

fn interp_matrix3(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3], g: f32) -> [[f32; 3]; 3] {
    let mut r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][j] * g + b[i][j] * (1.0 - g);
        }
    }
    r
}

fn dcp_interp_g(profile: &DcpProfile, wb_coeffs: [f32; 4]) -> f32 {
    let Some(illum2) = profile.calibration_illuminant2 else {
        return 1.0;
    };
    let cct1 = dcp_illuminant_cct(profile.calibration_illuminant1);
    let cct2 = dcp_illuminant_cct(illum2);
    if (cct1 - cct2).abs() < 1.0 {
        return 1.0;
    }
    let scene = estimate_scene_cct(wb_coeffs, &pad4(profile.color_matrix1));
    let lo = cct1.min(cct2);
    let hi = cct1.max(cct2);
    let inv1 = 1.0 / cct1;
    let inv2 = 1.0 / cct2;
    let inv_scene = 1.0 / scene.clamp(lo, hi);
    ((inv_scene - inv2) / (inv1 - inv2)).clamp(0.0, 1.0)
}

pub fn dcp_weight(profile: &DcpProfile, wb_coeffs: [f32; 4], illuminant: DcpIlluminant) -> f32 {
    match illuminant {
        DcpIlluminant::First => 1.0,
        DcpIlluminant::Second if profile.is_dual_illuminant() => 0.0,
        DcpIlluminant::Second => 1.0,
        DcpIlluminant::Interpolated => dcp_interp_g(profile, wb_coeffs),
    }
}

pub fn dcp_cam_to_srgb(
    profile: &DcpProfile,
    wb_coeffs: [f32; 4],
    illuminant: DcpIlluminant,
) -> [[f32; 3]; 3] {
    let g = dcp_weight(profile, wb_coeffs, illuminant);
    if let Some(fm1) = profile.forward_matrix1 {
        let fm = match profile.forward_matrix2 {
            Some(fm2) => interp_matrix3(&fm1, &fm2, g),
            None => fm1,
        };
        let cat = bradford_cat(D50_XY, D65_XY);
        let cam_to_xyz65 = mat3_mul(&cat, &fm);
        mat3_mul(&XYZ_TO_SRGB_D65, &cam_to_xyz65)
    } else {
        let cm = match profile.color_matrix2 {
            Some(cm2) => interp_matrix3(&profile.color_matrix1, &cm2, g),
            None => profile.color_matrix1,
        };
        cam_to_srgb_matrix(pad4(cm))
    }
}

pub fn merge_huesat(a: &HueSatMap, b: &HueSatMap, g: f32) -> HueSatMap {
    if a.hue_div != b.hue_div
        || a.sat_div != b.sat_div
        || a.val_div != b.val_div
        || a.data.len() != b.data.len()
    {
        return a.clone();
    }
    let data = a
        .data
        .iter()
        .zip(b.data.iter())
        .map(|(x, y)| {
            [
                x[0] * g + y[0] * (1.0 - g),
                x[1] * g + y[1] * (1.0 - g),
                x[2] * g + y[2] * (1.0 - g),
            ]
        })
        .collect();
    HueSatMap {
        hue_div: a.hue_div,
        sat_div: a.sat_div,
        val_div: a.val_div,
        encoding: a.encoding,
        data,
    }
}
fn sample_huesat(map: &HueSatMap, hsv: [f32; 3]) -> [f32; 3] {
    let hue_div = map.hue_div as usize;
    let sat_div = map.sat_div as usize;
    let val_div = map.val_div.max(1) as usize;

    let h_scaled = hsv[0] / 6.0 * hue_div as f32;
    let s_scaled = hsv[1] * (sat_div - 1).max(1) as f32;
    let v_scaled = if val_div > 1 {
        hsv[2].clamp(0.0, 1.0) * (val_div - 1) as f32
    } else {
        0.0
    };

    let h0 = h_scaled.floor();
    let hf = h_scaled - h0;
    let h0 = (h0 as usize).rem_euclid(hue_div);
    let h1 = (h0 + 1).rem_euclid(hue_div);

    let s0f = s_scaled.floor().min((sat_div - 1) as f32);
    let sf = (s_scaled - s0f).clamp(0.0, 1.0);
    let s0 = s0f as usize;
    let s1 = (s0 + 1).min(sat_div - 1);

    let v0f = v_scaled.floor().min((val_div - 1) as f32);
    let vf = (v_scaled - v0f).clamp(0.0, 1.0);
    let v0 = v0f as usize;
    let v1 = (v0 + 1).min(val_div - 1);

    let idx =
        |h: usize, s: usize, v: usize| -> [f32; 3] { map.data[(v * sat_div + s) * hue_div + h] };
    let lerp = |a: [f32; 3], b: [f32; 3], t: f32| -> [f32; 3] {
        [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]
    };

    let c000 = idx(h0, s0, v0);
    let c100 = idx(h1, s0, v0);
    let c010 = idx(h0, s1, v0);
    let c110 = idx(h1, s1, v0);
    let c00 = lerp(c000, c100, hf);
    let c10 = lerp(c010, c110, hf);
    let cv0 = lerp(c00, c10, sf);
    if val_div <= 1 {
        return cv0;
    }
    let c001 = idx(h0, s0, v1);
    let c101 = idx(h1, s0, v1);
    let c011 = idx(h0, s1, v1);
    let c111 = idx(h1, s1, v1);
    let c01 = lerp(c001, c101, hf);
    let c11 = lerp(c011, c111, hf);
    let cv1 = lerp(c01, c11, sf);
    lerp(cv0, cv1, vf)
}

pub fn apply_huesat(
    map: &HueSatMap,
    to_pp: &[[f32; 3]; 3],
    from_pp: &[[f32; 3]; 3],
    rgb: [f32; 3],
) -> [f32; 3] {
    let pp = mat3_vec(to_pp, rgb);
    let out_pp = apply_huesat_in_space(map, pp, false);
    mat3_vec(from_pp, out_pp)
}

pub(super) fn apply_huesat_in_space(map: &HueSatMap, rgb: [f32; 3], bounded: bool) -> [f32; 3] {
    if rgb.iter().any(|v| !v.is_finite()) {
        return rgb;
    }
    if !bounded && rgb.iter().any(|v| *v < 0.0) {
        return rgb;
    }
    if bounded && !rgb.iter().any(|v| (0.0..=1.0).contains(v)) {
        return rgb;
    }
    let source = if bounded {
        [
            rgb[0].clamp(0.0, 1.0),
            rgb[1].clamp(0.0, 1.0),
            rgb[2].clamp(0.0, 1.0),
        ]
    } else {
        rgb
    };
    let mut hsv = rgb_to_hsv(source);
    let encoded_v = match map.encoding {
        HsvEncoding::Srgb => srgb_gamma(hsv[2].clamp(0.0, 1.0)),
        HsvEncoding::Linear => hsv[2],
    };
    let delta = sample_huesat(map, [hsv[0], hsv[1], encoded_v]);
    hsv[0] = (hsv[0] + delta[0] / 60.0).rem_euclid(6.0);
    hsv[1] *= delta[1];
    hsv[2] = match map.encoding {
        HsvEncoding::Srgb => srgb_degamma((encoded_v * delta[2]).clamp(0.0, 1.0)),
        HsvEncoding::Linear => hsv[2] * delta[2],
    };
    if bounded {
        hsv[1] = hsv[1].clamp(0.0, 1.0);
        hsv[2] = hsv[2].clamp(0.0, 1.0);
    }
    let out = hsv_to_rgb(hsv);
    if out.iter().all(|v| v.is_finite()) {
        out
    } else {
        rgb
    }
}

pub(super) fn apply_profile_tone_curve(curve: &[[f32; 2]], rgb: [f32; 3]) -> [f32; 3] {
    if !rgb.iter().any(|v| (0.0..=1.0).contains(v)) {
        return rgb;
    }
    let source = [
        rgb[0].clamp(0.0, 1.0),
        rgb[1].clamp(0.0, 1.0),
        rgb[2].clamp(0.0, 1.0),
    ];
    let mut order = [0usize, 1, 2];
    order.sort_by(|a, b| source[*a].total_cmp(&source[*b]));
    let lo = source[order[0]];
    let mid = source[order[1]];
    let hi = source[order[2]];
    let lo_out = eval_tone_curve(curve, lo);
    let hi_out = eval_tone_curve(curve, hi);
    let mid_out = if hi - lo <= 1e-8 {
        lo_out
    } else {
        lo_out + (hi_out - lo_out) * (mid - lo) / (hi - lo)
    };
    let mut out = [0.0f32; 3];
    out[order[0]] = lo_out;
    out[order[1]] = mid_out;
    out[order[2]] = hi_out;
    out
}

pub fn apply_dcp_finish(
    look: Option<&HueSatMap>,
    tone_curve: Option<&[[f32; 2]]>,
    to_pp: &[[f32; 3]; 3],
    from_pp: &[[f32; 3]; 3],
    rgb: [f32; 3],
) -> [f32; 3] {
    let mut pp = mat3_vec(to_pp, rgb);
    if let Some(map) = look {
        pp = apply_huesat_in_space(map, pp, true);
    }
    if let Some(curve) = tone_curve {
        pp = apply_profile_tone_curve(curve, pp);
    }
    mat3_vec(from_pp, pp)
}

pub const DCP_FALLBACK_TONE_CURVE: [[f32; 2]; 13] = [
    [0.0, 0.0],
    [0.05, 0.028_838],
    [0.1, 0.061_36],
    [0.15, 0.101_695],
    [0.2, 0.152_542],
    [0.3, 0.288_136],
    [0.4, 0.436_441],
    [0.5, 0.578_814],
    [0.6, 0.703_39],
    [0.7, 0.805_085],
    [0.8, 0.889_831],
    [0.9, 0.954_802],
    [1.0, 1.0],
];

pub const DEFAULT_COLOR_TONE_CURVE: [[f32; 2]; 13] = [
    [0.0, 0.0],
    [0.05, 0.028_838],
    [0.1, 0.061_36],
    [0.15, 0.101_695],
    [0.2, 0.152_542],
    [0.3, 0.288_136],
    [0.4, 0.436_441],
    [0.5, 0.578_814],
    [0.6, 0.703_39],
    [0.7, 0.805_085],
    [0.8, 0.889_831],
    [0.9, 0.954_802],
    [1.0, 1.0],
];

pub fn eval_tone_curve(curve: &[[f32; 2]], x: f32) -> f32 {
    if curve.len() < 2 {
        return x;
    }
    let x = x.clamp(0.0, 1.0);
    if x <= curve[0][0] {
        return curve[0][1];
    }
    if x >= curve[curve.len() - 1][0] {
        return curve[curve.len() - 1][1];
    }
    let hi = curve.partition_point(|p| p[0] < x).max(1);
    let a = curve[hi - 1];
    let b = curve[hi];
    let span = b[0] - a[0];
    if span <= 1e-9 {
        return a[1];
    }
    let t = (x - a[0]) / span;
    a[1] + (b[1] - a[1]) * t
}
