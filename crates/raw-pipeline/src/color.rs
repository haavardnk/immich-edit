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
