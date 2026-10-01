use super::*;

const OKLAB_A_WEIGHTS: [f32; 3] = [1.977_998_5, -2.428_592_2, 0.450_593_7];
const OKLAB_B_WEIGHTS: [f32; 3] = [0.025_904_037, 0.782_771_77, -0.808_675_77];

const SPACES: [OutputColorSpace; 2] = [OutputColorSpace::SRgb, OutputColorSpace::DisplayP3];

fn dot(w: [f32; 3], v: [f32; 3]) -> f32 {
    w[0] * v[0] + w[1] * v[1] + w[2] * v[2]
}

fn oklab(srgb: [f32; 3]) -> [f32; 3] {
    let lms = mat3_vec(&SRGB_TO_LMS, srgb).map(f32::cbrt);
    [
        dot(OKLAB_L_WEIGHTS, lms),
        dot(OKLAB_A_WEIGHTS, lms),
        dot(OKLAB_B_WEIGHTS, lms),
    ]
}

fn to_space(srgb: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    match cs {
        OutputColorSpace::SRgb => srgb,
        OutputColorSpace::DisplayP3 => mat3_vec(&SRGB_LINEAR_TO_DISPLAY_P3, srgb),
    }
}

fn to_srgb(c: [f32; 3], cs: OutputColorSpace) -> [f32; 3] {
    match cs {
        OutputColorSpace::SRgb => c,
        OutputColorSpace::DisplayP3 => mat3_vec(&DISPLAY_P3_TO_SRGB_LINEAR, c),
    }
}

#[test]
fn in_gamut_colours_pass_through() {
    for cs in SPACES {
        for c in [
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.8, 0.1, 0.4],
            [0.02, 0.6, 1.0],
        ] {
            let out = map_to_gamut(c, cs);
            if out != c {
                panic!("{cs:?} {c:?} is in gamut and must not move, got {out:?}");
            }
        }
    }
}

#[test]
fn out_of_gamut_colours_keep_oklab_hue_and_lightness() {
    let samples = [
        [-0.072_8, -0.008_3, 1.118_7],
        [0.1, 0.2, 1.5],
        [-0.2, 0.5, 0.9],
        [1.5, 0.4, 0.2],
        [0.0, 1.4, 0.1],
        [1.2, -0.1, 0.6],
        [0.9, 0.95, -0.3],
        [1.6, 0.2, 1.3],
    ];
    for (cs, srgb) in SPACES.into_iter().flat_map(|cs| samples.map(|s| (cs, s))) {
        let c = to_space(srgb, cs);
        let out = map_to_gamut(c, cs);
        if out.iter().any(|v| !(-1e-5..=1.0 + 1e-5).contains(v)) {
            panic!("{cs:?} {srgb:?} must land in the unit cube, got {out:?}");
        }
        if out.iter().all(|v| (1e-3..=1.0 - 1e-3).contains(v)) {
            panic!("{cs:?} {srgb:?} must stop on the gamut boundary, got {out:?}");
        }
        let [l_in, a_in, b_in] = oklab(srgb);
        let [l_out, a_out, b_out] = oklab(to_srgb(out, cs));
        if (l_out - l_in.clamp(0.0, 1.0)).abs() > 2e-3 {
            panic!("{cs:?} {srgb:?} lightness moved: {l_in} -> {l_out}");
        }
        let dh = (b_out.atan2(a_out) - b_in.atan2(a_in)).to_degrees();
        let dh = (dh + 180.0).rem_euclid(360.0) - 180.0;
        if dh.abs() > 0.5 {
            panic!("{cs:?} {srgb:?} hue moved {dh} degrees");
        }
    }
}

#[test]
fn above_white_lightness_lands_on_white() {
    for cs in SPACES {
        let out = map_to_gamut(to_space([2.0, 1.0, 0.3], cs), cs);
        if out.iter().any(|v| (v - 1.0).abs() > 1e-4) {
            panic!("{cs:?} Oklab L above 1 must map to white, got {out:?}");
        }
    }
}
