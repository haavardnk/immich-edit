pub const OETF_LUT_SIZE: usize = 4096;

pub const RAW_SENSOR_WHITE: f32 = 1.0;
pub const RAW_LINEAR_CEILING: f32 = 4.0;
pub const HL_RECONSTRUCT_KNEE: f32 = 0.92;
pub const HL_RECONSTRUCT_BIAS: f32 = 0.1;

pub const LUMA_R: f32 = 0.2126;
pub const LUMA_G: f32 = 0.7152;
pub const LUMA_B: f32 = 0.0722;

pub const SRGB_OETF_LINEAR_CUTOFF: f32 = 0.003_130_8;
pub const SRGB_OETF_LINEAR_SLOPE: f32 = 12.92;
pub const SRGB_OETF_GAMMA_SCALE: f32 = 1.055;
pub const SRGB_OETF_GAMMA_OFFSET: f32 = 0.055;
pub const SRGB_OETF_GAMMA: f32 = 1.0 / 2.4;
