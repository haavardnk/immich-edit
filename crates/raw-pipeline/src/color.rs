pub const XYZ_TO_SRGB_D65: [[f32; 3]; 3] = [
    [3.240_454, -1.537_138_5, -0.498_531_4],
    [-0.969_266, 1.876_010_8, 0.041_556],
    [0.055_643_4, -0.204_025_9, 1.057_225_2],
];

pub fn cam_to_srgb_matrix(cam_to_xyz: [[f32; 4]; 3]) -> [[f32; 3]; 3] {
    let mut cam_xyz = [[0.0f32; 3]; 3];
    for (i, row) in cam_xyz.iter_mut().enumerate() {
        for (j, slot) in row.iter_mut().enumerate() {
            *slot = cam_to_xyz[i][j];
        }
    }
    let mut out = [[0.0f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0f32;
            for k in 0..3 {
                sum += XYZ_TO_SRGB_D65[i][k] * cam_xyz[k][j];
            }
            out[i][j] = sum;
        }
    }
    out
}

pub fn identity_3x3() -> [[f32; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

pub fn is_zero_matrix(m: &[[f32; 4]; 3]) -> bool {
    m.iter().all(|row| row.iter().all(|v| v.abs() < 1e-6))
}
