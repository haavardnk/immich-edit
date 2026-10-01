use std::sync::LazyLock;

use crate::color::{DISPLAY_P3_TO_SRGB_LINEAR, SRGB_LINEAR_TO_DISPLAY_P3, mat3_mul, mat3_vec};
use crate::frame::OutputColorSpace;

pub const GAMUT_SEARCH_STEPS: u32 = 16;

pub const OKLAB_L_WEIGHTS: [f32; 3] = [0.210_454_26, 0.793_617_8, -0.004_072_047];

const SRGB_TO_LMS: [[f32; 3]; 3] = [
    [0.412_221_46, 0.536_332_55, 0.051_445_995],
    [0.211_903_5, 0.680_699_5, 0.107_396_96],
    [0.088_302_46, 0.281_718_85, 0.629_978_7],
];

const LMS_TO_SRGB: [[f32; 3]; 3] = [
    [4.076_741_7, -3.307_711_6, 0.230_969_94],
    [-1.268_438, 2.609_757_4, -0.341_319_38],
    [-0.004_196_086_3, -0.703_418_6, 1.707_614_7],
];

pub struct LmsBasis {
    pub to_lms: [[f32; 3]; 3],
    pub from_lms: [[f32; 3]; 3],
}

pub fn lms_basis(cs: OutputColorSpace) -> &'static LmsBasis {
    static SRGB: LmsBasis = LmsBasis {
        to_lms: SRGB_TO_LMS,
        from_lms: LMS_TO_SRGB,
    };
    static P3: LazyLock<LmsBasis> = LazyLock::new(|| LmsBasis {
        to_lms: mat3_mul(&SRGB_TO_LMS, &DISPLAY_P3_TO_SRGB_LINEAR),
        from_lms: mat3_mul(&SRGB_LINEAR_TO_DISPLAY_P3, &LMS_TO_SRGB),
    });
    match cs {
        OutputColorSpace::SRgb => &SRGB,
        OutputColorSpace::DisplayP3 => &P3,
    }
}

fn in_unit_cube(c: [f32; 3]) -> bool {
    c.iter().all(|v| (0.0..=1.0).contains(v))
}

pub fn map_to_gamut(c: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    if in_unit_cube(c) {
        return c;
    }
    let basis = lms_basis(cs);
    let lms = mat3_vec(&basis.to_lms, c).map(f32::cbrt);
    let l0 =
        (OKLAB_L_WEIGHTS[0] * lms[0] + OKLAB_L_WEIGHTS[1] * lms[1] + OKLAB_L_WEIGHTS[2] * lms[2])
            .clamp(0.0, 1.0);
    let at = |t: f32| {
        let cube = lms.map(|v| {
            let x = l0 + t * (v - l0);
            x * x * x
        });
        mat3_vec(&basis.from_lms, cube)
    };
    let mut lo = 0.0f32;
    let mut hi = 1.0f32;
    for _ in 0..GAMUT_SEARCH_STEPS {
        let mid = 0.5 * (lo + hi);
        if in_unit_cube(at(mid)) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at(lo)
}

#[cfg(test)]
mod tests;
