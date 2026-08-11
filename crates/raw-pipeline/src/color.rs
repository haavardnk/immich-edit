pub const XYZ_TO_SRGB_D65: [[f32; 3]; 3] = [
    [3.240_454, -1.537_138_5, -0.498_531_4],
    [-0.969_266, 1.876_010_8, 0.041_556],
    [0.055_643_4, -0.204_025_9, 1.057_225_2],
];

pub const SRGB_TO_XYZ_D65: [[f32; 3]; 3] = [
    [0.412_456_4, 0.357_576_1, 0.180_437_5],
    [0.212_672_9, 0.715_152_2, 0.072_175_0],
    [0.019_333_9, 0.119_192, 0.950_304_1],
];

pub const SRGB_LINEAR_TO_DISPLAY_P3: [[f32; 3]; 3] = [
    [0.822_461_97, 0.177_538_03, 0.0],
    [0.033_194_2, 0.966_805_8, 0.0],
    [0.017_082_63, 0.072_397_07, 0.910_520_3],
];

pub const DISPLAY_P3_TO_SRGB_LINEAR: [[f32; 3]; 3] = [
    [1.224_940_2, -0.224_940_18, 0.0],
    [-0.042_056_96, 1.042_057, 0.0],
    [-0.019_637_56, -0.078_636_04, 1.098_273_6],
];

const BRADFORD: [[f32; 3]; 3] = [
    [0.8951, 0.2664, -0.1614],
    [-0.7502, 1.7135, 0.0367],
    [0.0389, -0.0685, 1.0296],
];

const BRADFORD_INV: [[f32; 3]; 3] = [
    [0.986_993, -0.147_054_3, 0.159_962_7],
    [0.432_305_3, 0.518_360_3, 0.049_291_2],
    [-0.008_528_7, 0.040_042_8, 0.968_486_7],
];

const D65_XY: (f32, f32) = (0.312_71, 0.329_02);

fn mat3_mul(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut r = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    r
}

fn mat3_vec(m: &[[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn cct_to_xy(cct: f32) -> (f32, f32) {
    let t = cct;
    let t2 = t * t;
    let t3 = t2 * t;
    let x = if t <= 7000.0 {
        -4.607e9 / t3 + 2.9678e6 / t2 + 0.09911e3 / t + 0.244_063
    } else {
        -2.0064e9 / t3 + 1.9018e6 / t2 + 0.24748e3 / t + 0.237_040
    };
    let y = -3.0 * x * x + 2.87 * x - 0.275;
    (x, y)
}

fn bradford_cat(src_xy: (f32, f32), dst_xy: (f32, f32)) -> [[f32; 3]; 3] {
    let src_xyz = [
        src_xy.0 / src_xy.1,
        1.0,
        (1.0 - src_xy.0 - src_xy.1) / src_xy.1,
    ];
    let dst_xyz = [
        dst_xy.0 / dst_xy.1,
        1.0,
        (1.0 - dst_xy.0 - dst_xy.1) / dst_xy.1,
    ];
    let sc = mat3_vec(&BRADFORD, src_xyz);
    let dc = mat3_vec(&BRADFORD, dst_xyz);
    let diag_brad = [
        [
            dc[0] / sc[0] * BRADFORD[0][0],
            dc[0] / sc[0] * BRADFORD[0][1],
            dc[0] / sc[0] * BRADFORD[0][2],
        ],
        [
            dc[1] / sc[1] * BRADFORD[1][0],
            dc[1] / sc[1] * BRADFORD[1][1],
            dc[1] / sc[1] * BRADFORD[1][2],
        ],
        [
            dc[2] / sc[2] * BRADFORD[2][0],
            dc[2] / sc[2] * BRADFORD[2][1],
            dc[2] / sc[2] * BRADFORD[2][2],
        ],
    ];
    mat3_mul(&BRADFORD_INV, &diag_brad)
}

pub fn user_wb_matrix(temp: f64, tint: f64) -> [[f32; 3]; 3] {
    let target_cct = (6500.0 * 2.0_f64.powf(-temp / 100.0)).clamp(2000.0, 25000.0) as f32;
    let (dst_x, mut dst_y) = cct_to_xy(target_cct);
    dst_y += tint as f32 * 0.02 / 100.0;
    let cat_xyz = bradford_cat(D65_XY, (dst_x, dst_y));
    let tmp = mat3_mul(&cat_xyz, &SRGB_TO_XYZ_D65);
    mat3_mul(&XYZ_TO_SRGB_D65, &tmp)
}

pub fn estimate_scene_cct(wb_coeffs: [f32; 4], xyz_to_cam: &[[f32; 3]; 4]) -> f32 {
    let neutral = [
        1.0 / wb_coeffs[0].max(1e-6),
        1.0 / wb_coeffs[1].max(1e-6),
        1.0 / wb_coeffs[2].max(1e-6),
    ];
    let cam_3x3: [[f32; 3]; 3] = [xyz_to_cam[0], xyz_to_cam[1], xyz_to_cam[2]];
    if let Some(xyz_from_cam) = inverse_3x3(cam_3x3) {
        let xyz = mat3_vec(&xyz_from_cam, neutral);
        let sum = xyz[0] + xyz[1] + xyz[2];
        if sum > 1e-6 {
            let x = xyz[0] / sum;
            let y = xyz[1] / sum;
            let n = (x - 0.3320) / (0.1858 - y);
            let cct = 449.0 * n * n * n + 3525.0 * n * n + 6823.3 * n + 5520.33;
            return cct.clamp(2000.0, 25000.0);
        }
    }
    6504.0
}

pub fn resolve_xyz_to_cam(
    matrices: &[(f32, [[f32; 3]; 4])],
    wb_coeffs: [f32; 4],
    fallback: [[f32; 3]; 4],
) -> [[f32; 3]; 4] {
    let Some(last) = matrices.last() else {
        return fallback;
    };
    if matrices.len() < 2 {
        return fallback;
    }
    let cct = estimate_scene_cct(wb_coeffs, &last.1);
    interpolate_xyz_to_cam(matrices, cct)
}

pub fn interpolate_xyz_to_cam(matrices: &[(f32, [[f32; 3]; 4])], scene_cct: f32) -> [[f32; 3]; 4] {
    if matrices.len() < 2 {
        return matrices.first().map(|m| m.1).unwrap_or([[0.0; 3]; 4]);
    }
    let (cct_lo, m_lo) = matrices[0];
    let (cct_hi, m_hi) = matrices[matrices.len() - 1];
    let inv_lo = 1.0 / cct_lo;
    let inv_hi = 1.0 / cct_hi;
    let inv_scene = 1.0 / scene_cct.clamp(cct_lo, cct_hi);
    let t = if (inv_lo - inv_hi).abs() > 1e-9 {
        (inv_scene - inv_hi) / (inv_lo - inv_hi)
    } else {
        0.5
    };
    let mut result = [[0.0f32; 3]; 4];
    for i in 0..4 {
        for j in 0..3 {
            result[i][j] = m_lo[i][j] * t + m_hi[i][j] * (1.0 - t);
        }
    }
    result
}

fn inverse_3x3(m: [[f32; 3]; 3]) -> Option<[[f32; 3]; 3]> {
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-9 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        [
            (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det,
        ],
        [
            (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det,
            (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det,
        ],
        [
            (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det,
            (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det,
            (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det,
        ],
    ])
}

pub fn cam_to_srgb_matrix(xyz_to_cam: [[f32; 3]; 4]) -> [[f32; 3]; 3] {
    let mut srgb_to_cam = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0f32;
            for k in 0..3 {
                sum += xyz_to_cam[i][k] * SRGB_TO_XYZ_D65[k][j];
            }
            srgb_to_cam[i][j] = sum;
        }
    }
    for row in &mut srgb_to_cam {
        let s = row[0] + row[1] + row[2];
        if s.abs() > 1e-9 {
            row[0] /= s;
            row[1] /= s;
            row[2] /= s;
        }
    }
    inverse_3x3(srgb_to_cam).unwrap_or_else(identity_3x3)
}

pub fn identity_3x3() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

pub fn srgb_lin_to_display_p3(rgb: [f32; 3]) -> [f32; 3] {
    mat3_vec(&SRGB_LINEAR_TO_DISPLAY_P3, rgb)
}

pub fn display_p3_to_srgb_lin(rgb: [f32; 3]) -> [f32; 3] {
    mat3_vec(&DISPLAY_P3_TO_SRGB_LINEAR, rgb)
}

pub fn is_unusable_matrix(m: &[[f32; 3]; 4]) -> bool {
    m.iter().any(|row| row.iter().any(|v| !v.is_finite()))
        || m.iter()
            .take(3)
            .all(|row| row.iter().all(|v| v.abs() < 1e-6))
}

// ---- DCP (DNG camera profile) color math ----

use crate::dcp::{DcpIlluminant, DcpProfile, HsvEncoding, HueSatMap};

const D50_XY: (f32, f32) = (0.345_67, 0.358_50);

const PROPHOTO_FROM_XYZ_D50: [[f32; 3]; 3] = [
    [1.345_943_3, -0.255_607_5, -0.051_111_8],
    [-0.544_598_9, 1.508_167_3, 0.020_535_1],
    [0.0, 0.0, 1.211_812_8],
];

const XYZ_D50_FROM_PROPHOTO: [[f32; 3]; 3] = [
    [0.797_674_9, 0.135_191_7, 0.031_353_4],
    [0.288_040_2, 0.711_874_1, 0.000_085_7],
    [0.0, 0.0, 0.825_21],
];

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

pub fn srgb_lin_to_prophoto_matrix() -> [[f32; 3]; 3] {
    let cat = bradford_cat(D65_XY, D50_XY);
    let xyz50 = mat3_mul(&cat, &SRGB_TO_XYZ_D65);
    mat3_mul(&PROPHOTO_FROM_XYZ_D50, &xyz50)
}

pub fn prophoto_to_srgb_lin_matrix() -> [[f32; 3]; 3] {
    let cat = bradford_cat(D50_XY, D65_XY);
    let xyz65 = mat3_mul(&cat, &XYZ_D50_FROM_PROPHOTO);
    mat3_mul(&XYZ_TO_SRGB_D65, &xyz65)
}

fn rgb_to_hsv(rgb: [f32; 3]) -> [f32; 3] {
    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let mut h = if delta <= 0.0 {
        0.0
    } else if max == r {
        ((g - b) / delta).rem_euclid(6.0)
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    if h < 0.0 {
        h += 6.0;
    }
    let s = if max <= 0.0 { 0.0 } else { delta / max };
    [h, s, max]
}

fn hsv_to_rgb(hsv: [f32; 3]) -> [f32; 3] {
    let h = hsv[0].rem_euclid(6.0);
    let s = hsv[1].clamp(0.0, 1.0);
    let v = hsv[2];
    if s <= 0.0 {
        return [v, v, v];
    }
    let i = h.floor();
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    match i as i32 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

fn srgb_gamma(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        12.92 * c
    } else {
        1.055 * c.max(0.0).powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_degamma(c: f32) -> f32 {
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
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

fn apply_huesat_in_space(map: &HueSatMap, rgb: [f32; 3], bounded: bool) -> [f32; 3] {
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

fn apply_profile_tone_curve(curve: &[[f32; 2]], rgb: [f32; 3]) -> [f32; 3] {
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

#[cfg(test)]
mod tests {
    use super::*;

    const MATRIX_A: [[f32; 3]; 4] = [
        [0.8, 0.1, 0.1],
        [0.05, 0.9, 0.05],
        [0.1, 0.2, 0.7],
        [0.0, 0.0, 0.0],
    ];
    const MATRIX_D65: [[f32; 3]; 4] = [
        [0.6, 0.3, 0.1],
        [0.1, 0.8, 0.1],
        [0.05, 0.1, 0.85],
        [0.0, 0.0, 0.0],
    ];

    #[test]
    fn display_p3_roundtrip_is_identity() {
        for c in [[0.2, 0.5, 0.8], [1.0, 0.0, 0.0], [0.3, 0.9, 0.1]] {
            let back = display_p3_to_srgb_lin(srgb_lin_to_display_p3(c));
            for i in 0..3 {
                assert!((back[i] - c[i]).abs() < 1e-4, "roundtrip drift at {i}");
            }
        }
    }

    #[test]
    fn srgb_red_maps_inside_display_p3() {
        let p3 = srgb_lin_to_display_p3([1.0, 0.0, 0.0]);
        assert!(p3.iter().all(|&v| (0.0..=1.0).contains(&v)));
        assert!(p3[0] < 1.0 && p3[0] > 0.8);
    }

    #[test]
    fn display_p3_red_is_outside_srgb() {
        let srgb = display_p3_to_srgb_lin([1.0, 0.0, 0.0]);
        assert!(srgb[0] > 1.0 || srgb[1] < 0.0 || srgb[2] < 0.0);
    }

    #[test]
    fn interpolate_at_low_cct_returns_warm_matrix() {
        let matrices = vec![(2856.0, MATRIX_A), (6504.0, MATRIX_D65)];
        let result = interpolate_xyz_to_cam(&matrices, 2856.0);
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (result[i][j] - MATRIX_A[i][j]).abs() < 1e-5,
                    "mismatch at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn interpolate_at_high_cct_returns_cool_matrix() {
        let matrices = vec![(2856.0, MATRIX_A), (6504.0, MATRIX_D65)];
        let result = interpolate_xyz_to_cam(&matrices, 6504.0);
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    (result[i][j] - MATRIX_D65[i][j]).abs() < 1e-5,
                    "mismatch at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn interpolate_midpoint_blends() {
        let matrices = vec![(2856.0, MATRIX_A), (6504.0, MATRIX_D65)];
        let mid_cct = 4000.0;
        let result = interpolate_xyz_to_cam(&matrices, mid_cct);
        for i in 0..3 {
            for j in 0..3 {
                assert!(
                    result[i][j] > MATRIX_A[i][j].min(MATRIX_D65[i][j]) - 1e-5
                        && result[i][j] < MATRIX_A[i][j].max(MATRIX_D65[i][j]) + 1e-5,
                    "out of range at [{i}][{j}]: {}",
                    result[i][j]
                );
            }
        }
    }

    #[test]
    fn interpolate_single_matrix_returns_it() {
        let matrices = vec![(6504.0, MATRIX_D65)];
        let result = interpolate_xyz_to_cam(&matrices, 4000.0);
        assert_eq!(result, MATRIX_D65);
    }

    #[test]
    fn estimate_cct_returns_valid_range() {
        let matrix: [[f32; 3]; 4] = [
            [0.8, 0.1, 0.1],
            [0.05, 0.9, 0.05],
            [0.1, 0.2, 0.7],
            [0.0, 0.0, 0.0],
        ];
        let wb = [2.0, 1.0, 1.5, 1.0];
        let cct = estimate_scene_cct(wb, &matrix);
        assert!((2000.0..=25000.0).contains(&cct), "cct={cct} out of range");
    }

    fn make_dcp(fm: Option<[[f32; 3]; 3]>) -> DcpProfile {
        DcpProfile {
            name: None,
            copyright: None,
            unique_camera_model: None,
            calibration_illuminant1: 21,
            calibration_illuminant2: None,
            color_matrix1: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            color_matrix2: None,
            forward_matrix1: fm,
            forward_matrix2: None,
            huesatmap1: None,
            huesatmap2: None,
            look_table: None,
            tone_curve: None,
            baseline_exposure_offset: 0.0,
            default_black_render: 0,
            embed_policy: 0,
        }
    }

    #[test]
    fn dcp_forward_matrix_maps_neutral_to_white() {
        // ForwardMatrix rows sum to D50 whitepoint XYZ, per DNG spec.
        let fm = [
            [0.9642 * 0.5, 0.9642 * 0.3, 0.9642 * 0.2],
            [1.0 * 0.4, 1.0 * 0.35, 1.0 * 0.25],
            [0.8249 * 0.1, 0.8249 * 0.3, 0.8249 * 0.6],
        ];
        let profile = make_dcp(Some(fm));
        let m = dcp_cam_to_srgb(&profile, [2.0, 1.0, 1.5, 1.0], DcpIlluminant::Interpolated);
        let out = mat3_vec(&m, [1.0, 1.0, 1.0]);
        for c in out {
            assert!((c - 1.0).abs() < 1e-3, "neutral -> {out:?}");
        }
    }

    #[test]
    fn prophoto_roundtrip_is_identity() {
        let to_pp = srgb_lin_to_prophoto_matrix();
        let from_pp = prophoto_to_srgb_lin_matrix();
        for rgb in [[0.2, 0.5, 0.8], [0.9, 0.1, 0.3], [0.5, 0.5, 0.5]] {
            let round = mat3_vec(&from_pp, mat3_vec(&to_pp, rgb));
            for (a, b) in round.iter().zip(rgb.iter()) {
                assert!((a - b).abs() < 1e-3, "roundtrip {round:?} vs {rgb:?}");
            }
        }
    }

    #[test]
    fn huesat_identity_map_preserves_color() {
        let map = HueSatMap {
            hue_div: 4,
            sat_div: 3,
            val_div: 1,
            encoding: HsvEncoding::Linear,
            data: vec![[0.0, 1.0, 1.0]; 12],
        };
        let to_pp = srgb_lin_to_prophoto_matrix();
        let from_pp = prophoto_to_srgb_lin_matrix();
        for rgb in [[0.3, 0.5, 0.7], [0.8, 0.2, 0.4]] {
            let out = apply_huesat(&map, &to_pp, &from_pp, rgb);
            for (a, b) in out.iter().zip(rgb.iter()) {
                assert!((a - b).abs() < 2e-3, "huesat identity {out:?} vs {rgb:?}");
            }
        }
    }

    #[test]
    fn srgb_encoded_huesat_identity_preserves_color() {
        let map = HueSatMap {
            hue_div: 4,
            sat_div: 3,
            val_div: 3,
            encoding: HsvEncoding::Srgb,
            data: vec![[0.0, 1.0, 1.0]; 36],
        };
        let to_pp = srgb_lin_to_prophoto_matrix();
        let from_pp = prophoto_to_srgb_lin_matrix();
        let rgb = [0.3, 0.5, 0.7];
        let out = apply_huesat(&map, &to_pp, &from_pp, rgb);
        if out
            .iter()
            .zip(rgb.iter())
            .any(|(a, b)| (a - b).abs() >= 2e-3)
        {
            panic!("sRGB huesat identity {out:?} vs {rgb:?}");
        }
    }

    #[test]
    fn dcp_finish_preserves_neutral_axis() {
        let map = HueSatMap {
            hue_div: 4,
            sat_div: 3,
            val_div: 3,
            encoding: HsvEncoding::Srgb,
            data: vec![[30.0, 1.5, 0.8]; 36],
        };
        let to_pp = srgb_lin_to_prophoto_matrix();
        let from_pp = prophoto_to_srgb_lin_matrix();
        let out = apply_dcp_finish(
            Some(&map),
            Some(&DCP_FALLBACK_TONE_CURVE),
            &to_pp,
            &from_pp,
            [0.4, 0.4, 0.4],
        );
        if (out[0] - out[1]).abs() > 2e-3 || (out[1] - out[2]).abs() > 2e-3 {
            panic!("DCP finish shifted neutral axis: {out:?}");
        }
    }

    #[test]
    fn dcp_finish_orders_looktable_before_tone_curve() {
        let look = HueSatMap {
            hue_div: 3,
            sat_div: 3,
            val_div: 1,
            encoding: HsvEncoding::Srgb,
            data: vec![[24.0, 1.3, 0.85]; 9],
        };
        let curve = [[0.0f32, 0.0], [0.25, 0.4], [0.6, 0.55], [1.0, 1.0]];
        let to_pp = srgb_lin_to_prophoto_matrix();
        let from_pp = prophoto_to_srgb_lin_matrix();
        let rgb = [0.62, 0.28, 0.16];

        let got = apply_dcp_finish(Some(&look), Some(&curve), &to_pp, &from_pp, rgb);

        let pp = mat3_vec(&to_pp, rgb);
        let correct = mat3_vec(
            &from_pp,
            apply_profile_tone_curve(&curve, apply_huesat_in_space(&look, pp, true)),
        );
        let wrong = mat3_vec(
            &from_pp,
            apply_huesat_in_space(&look, apply_profile_tone_curve(&curve, pp), true),
        );

        if (0..3).any(|i| (got[i] - correct[i]).abs() > 1e-6) {
            panic!("finish is not look-before-tone: {got:?} vs {correct:?}");
        }
        if (0..3).all(|i| (correct[i] - wrong[i]).abs() < 1e-4) {
            panic!("stage order is not observable; test cannot guard ordering");
        }
        let golden = [0.5450712, 0.38464665, 0.07066608];
        if (0..3).any(|i| (got[i] - golden[i]).abs() > 1e-4) {
            panic!("DCP finish golden drift: {got:?}");
        }
    }

    #[test]
    fn dcp_base_table_golden() {
        let map = HueSatMap {
            hue_div: 2,
            sat_div: 2,
            val_div: 1,
            encoding: HsvEncoding::Linear,
            data: vec![[18.0, 0.75, 1.08]; 4],
        };
        let to_pp = srgb_lin_to_prophoto_matrix();
        let from_pp = prophoto_to_srgb_lin_matrix();
        let got = apply_huesat(&map, &to_pp, &from_pp, [0.5, 0.24, 0.11]);
        let golden = [0.4618022, 0.35953194, 0.17766672];
        if (0..3).any(|i| (got[i] - golden[i]).abs() > 1e-4) {
            panic!("DCP base table golden drift: {got:?}");
        }
    }

    #[test]
    fn profile_tone_curve_preserves_hsv_hue() {
        let rgb = [0.72, 0.31, 0.18];
        let before = rgb_to_hsv(rgb);
        let out = apply_profile_tone_curve(&DCP_FALLBACK_TONE_CURVE, rgb);
        let after = rgb_to_hsv(out);
        if (before[0] - after[0]).abs() > 1e-5 {
            panic!("profile tone shifted hue: {before:?} -> {after:?}");
        }
    }

    #[test]
    fn tone_curve_is_monotonic_and_bounded() {
        let mut prev = -1.0;
        for i in 0..=20 {
            let x = i as f32 / 20.0;
            let y = eval_tone_curve(&DCP_FALLBACK_TONE_CURVE, x);
            assert!((0.0..=1.0).contains(&y));
            assert!(y >= prev - 1e-6, "not monotonic at {x}");
            prev = y;
        }
        assert!((eval_tone_curve(&DCP_FALLBACK_TONE_CURVE, 0.0)).abs() < 1e-6);
        assert!((eval_tone_curve(&DCP_FALLBACK_TONE_CURVE, 1.0) - 1.0).abs() < 1e-6);
    }
}
