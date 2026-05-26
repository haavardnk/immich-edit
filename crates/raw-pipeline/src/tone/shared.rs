pub const HIGHLIGHT_KNEE: f32 = 0.95;
pub const S_CURVE_BLEND: f32 = 0.15;
pub const OETF_LUT_SIZE: usize = 4096;

pub const SRGB_OETF_LINEAR_CUTOFF: f32 = 0.003_130_8;
pub const SRGB_OETF_LINEAR_SLOPE: f32 = 12.92;
pub const SRGB_OETF_GAMMA_SCALE: f32 = 1.055;
pub const SRGB_OETF_GAMMA_OFFSET: f32 = 0.055;
pub const SRGB_OETF_GAMMA: f32 = 1.0 / 2.4;

pub const AGX_MIN_EV: f32 = -10.0;
pub const AGX_MAX_EV: f32 = 6.5;
pub const AGX_MIN_LIN: f32 = 0.000_976_562_5;

pub const AGX_DESAT_KNEE: f32 = 0.7;
pub const AGX_DESAT_CEIL: f32 = 2.0;

pub const AGX_IN_R: [f32; 3] = [0.842_479, 0.078_411_03, 0.087_828_82];
pub const AGX_IN_G: [f32; 3] = [0.042_328_99, 0.878_869_2, 0.078_843_8];
pub const AGX_IN_B: [f32; 3] = [0.042_424_29, 0.078_843_8, 0.879_092_2];

pub const AGX_OUT_R: [f32; 3] = [1.196_799_4, -0.052_834_3, -0.143_965];
pub const AGX_OUT_G: [f32; 3] = [-0.058_063_2, 1.151_812_1, -0.093_748_94];
pub const AGX_OUT_B: [f32; 3] = [-0.086_345_55, -0.082_385_42, 1.168_730_9];

pub const AGX_SIGMOID_COEFFS: [f32; 8] = [
    -17.866_29,
    78.011_72,
    -126.701_2,
    92.060_05,
    -28.722_15,
    4.361_57,
    0.139_142_5,
    0.001_891_437_3,
];

pub const TONE_KIND_DEFAULT: u32 = 0;
pub const TONE_KIND_AGX: u32 = 1;
