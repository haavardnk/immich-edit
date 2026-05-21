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

pub fn is_unusable_matrix(m: &[[f32; 3]; 4]) -> bool {
    m.iter().any(|row| row.iter().any(|v| !v.is_finite()))
        || m.iter()
            .take(3)
            .all(|row| row.iter().all(|v| v.abs() < 1e-6))
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
}
