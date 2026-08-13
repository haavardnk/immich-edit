mod dcp;
mod hsv;
mod matrix;

#[cfg(test)]
mod tests;

pub use dcp::{
    DCP_FALLBACK_TONE_CURVE, DEFAULT_COLOR_TONE_CURVE, apply_dcp_finish, apply_huesat,
    dcp_cam_to_srgb, dcp_illuminant_cct, dcp_weight, eval_tone_curve, merge_huesat,
};
pub use matrix::{
    cam_to_srgb_matrix, display_p3_to_srgb_lin, estimate_scene_cct, identity_3x3,
    interpolate_xyz_to_cam, is_unusable_matrix, prophoto_to_srgb_lin_matrix, resolve_xyz_to_cam,
    srgb_lin_to_display_p3, srgb_lin_to_prophoto_matrix, user_wb_matrix,
};

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
