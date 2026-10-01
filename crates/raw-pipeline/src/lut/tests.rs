use super::*;
use crate::tone::srgb_oetf_scalar;

fn cube_entries(n: usize, f: impl Fn([f32; 3]) -> [f32; 3]) -> String {
    let last = (n - 1) as f32;
    (0..n * n * n)
        .map(|i| {
            let rgb = [i % n, (i / n) % n, i / (n * n)].map(|v| v as f32 / last);
            let [r, g, b] = f(rgb);
            format!("{r} {g} {b}\n")
        })
        .collect()
}

fn cube_source(header: &str, n: usize, f: impl Fn([f32; 3]) -> [f32; 3]) -> String {
    format!("{header}LUT_3D_SIZE {n}\n{}", cube_entries(n, f))
}

fn tint(rgb: [f32; 3]) -> [f32; 3] {
    [(rgb[0] * 1.1).min(1.0), rgb[1], rgb[2] * 0.85]
}

fn parse(src: &str) -> CubeLut {
    CubeLut::parse(src.as_bytes()).unwrap()
}

fn assert_close(got: [f32; 3], want: [f32; 3], tol: f32) {
    for c in 0..3 {
        assert!(
            (got[c] - want[c]).abs() < tol,
            "channel {c}: {got:?} vs {want:?}"
        );
    }
}

#[test]
fn identity_cube_is_neutral() {
    let lut = parse(&cube_source("TITLE \"x\"\n", 17, |c| c));
    assert_eq!(lut.cube().unwrap().size(), 17);
    assert!(lut.shaper().is_none());
    for probe in [[0.0; 3], [1.0; 3], [0.25, 0.5, 0.75], [0.1, 0.9, 0.3]] {
        assert_close(lut.sample(probe), probe, 1e-4);
    }
}

#[test]
fn inverting_cube_maps_corners() {
    let lut = parse(&cube_source("", 2, |c| c.map(|v| 1.0 - v)));
    assert_close(lut.sample([0.0; 3]), [1.0; 3], 1e-4);
    assert_close(lut.sample([1.0; 3]), [0.0; 3], 1e-4);
}

#[test]
fn cube_domain_maps_and_clamps() {
    for header in [
        "DOMAIN_MIN -1 -1 -1\nDOMAIN_MAX 3 3 3\n",
        "LUT_3D_INPUT_RANGE -1 3\n",
    ] {
        let lut = parse(&cube_source(header, 2, |c| c));
        let cube = lut.cube().unwrap();
        assert_eq!(cube.domain(), Domain::uniform([-1.0, 3.0]));
        for (input, want) in [
            (-9.0f32, 0.0f32),
            (-1.0, 0.0),
            (1.0, 0.5),
            (3.0, 1.0),
            (9.0, 1.0),
        ] {
            assert_close(lut.sample([input; 3]), [want; 3], 1e-4);
        }
    }
}

#[test]
fn shaper_feeds_the_cube() {
    let src = format!(
        "LUT_1D_SIZE 3\nLUT_3D_SIZE 2\nLUT_1D_INPUT_RANGE 0 4\nLUT_3D_INPUT_RANGE 0 1\n\
         0 0 0\n0.5 0.25 0.5\n1 1 1\n{}",
        cube_entries(2, |c| c.map(|v| 1.0 - v))
    );
    let lut = parse(&src);
    let shaper = lut.shaper().unwrap();
    assert_eq!(shaper.size(), 3);
    assert_eq!(shaper.domain(), Domain::uniform([0.0, 4.0]));
    assert_close(shaper.sample([1.0, 1.0, 3.0]), [0.25, 0.125, 0.75], 1e-6);
    assert_close(lut.sample([1.0, 1.0, 3.0]), [0.75, 0.875, 0.25], 1e-5);
}

#[test]
fn one_dimensional_lut_stands_alone() {
    let lut = parse("DOMAIN_MAX 2 2 2\nLUT_1D_SIZE 2\n1 1 1\n0 0 0\n");
    assert!(lut.cube().is_none());
    assert_eq!(lut.shaper().unwrap().domain().max, [2.0; 3]);
    assert_close(lut.sample([0.5, 1.0, 3.0]), [0.75, 0.5, 0.0], 1e-6);
}

#[test]
fn rejects_malformed_sources() {
    let ramp = cube_entries(2, |c| c);
    let cases: Vec<(String, LutParseError)> = vec![
        ("0 0 0\n1 1 1\n".into(), LutParseError::MissingSize),
        ("LUT_3D_SIZE 2\n".into(), LutParseError::Empty),
        (
            "LUT_3D_SIZE 2\n0 0 0\n1 1 1\n".into(),
            LutParseError::WrongEntryCount {
                expected: 8,
                found: 2,
            },
        ),
        (
            format!("LUT_1D_SIZE 2\nLUT_3D_SIZE 2\n{ramp}"),
            LutParseError::WrongEntryCount {
                expected: 10,
                found: 8,
            },
        ),
        (
            format!("LUT_3D_SIZE 2\nnan 0 0\n{}", "0 0 0\n".repeat(7)),
            LutParseError::NonFiniteValue,
        ),
        ("LUT_3D_SIZE 1\n0 0 0\n".into(), LutParseError::InvalidSize),
        ("LUT_3D_SIZE 66\n".into(), LutParseError::InvalidSize),
        ("LUT_1D_SIZE 65537\n".into(), LutParseError::InvalidSize),
        (
            "LUT_3D_SIZE 2\nLUT_3D_SIZE 2\n".into(),
            LutParseError::DuplicateDirective("LUT_3D_SIZE"),
        ),
        (
            "LUT_3D_INPUT_RANGE 0 1\nLUT_3D_INPUT_RANGE 0 1\n".into(),
            LutParseError::DuplicateDirective("LUT_3D_INPUT_RANGE"),
        ),
        (
            format!("DOMAIN_MIN 1 1 1\nDOMAIN_MAX 0 0 0\nLUT_3D_SIZE 2\n{ramp}"),
            LutParseError::InvalidDomain,
        ),
        (
            format!("LUT_3D_INPUT_RANGE 1 1\nLUT_3D_SIZE 2\n{ramp}"),
            LutParseError::InvalidDomain,
        ),
        (
            format!("DOMAIN_MAX 2 2 2\nLUT_3D_INPUT_RANGE 0 2\nLUT_3D_SIZE 2\n{ramp}"),
            LutParseError::ConflictingDomain,
        ),
        (
            format!("LUT_1D_INPUT_RANGE 0 2\nLUT_3D_SIZE 2\n{ramp}"),
            LutParseError::ConflictingDomain,
        ),
        (
            format!("DOMAIN_MAX 2 2 2\nLUT_1D_SIZE 2\nLUT_3D_SIZE 2\n0 0 0\n1 1 1\n{ramp}"),
            LutParseError::ConflictingDomain,
        ),
        (
            format!("LUT_IN_VIDEO_RANGE\nLUT_3D_SIZE 2\n{ramp}"),
            LutParseError::UnknownDirective("LUT_IN_VIDEO_RANGE".into()),
        ),
    ];
    for (src, want) in cases {
        assert_eq!(CubeLut::parse(src.as_bytes()), Err(want), "{src}");
    }
    let big = vec![b'0'; LUT_MAX_SOURCE_BYTES + 1];
    assert_eq!(CubeLut::parse(&big), Err(LutParseError::TooLarge));
}

#[test]
fn display_p3_grades_like_srgb() {
    let lut = parse(&cube_source("", 17, tint));
    let encode = |c: [f32; 3]| c.map(|v| srgb_oetf_scalar(v.clamp(0.0, 1.0)));
    for linear in [
        [0.18f32, 0.18, 0.18],
        [0.6, 0.2, 0.05],
        [0.05, 0.3, 0.7],
        [0.9, 0.85, 0.1],
    ] {
        for amount in [0.4f32, 1.0] {
            let srgb = lut.apply(encode(linear), amount, OutputColorSpace::SRgb);
            let want = encode(srgb_lin_to_display_p3(srgb.map(srgb_to_linear)));
            let p3 = encode(srgb_lin_to_display_p3(linear));
            let got = lut.apply(p3, amount, OutputColorSpace::DisplayP3);
            assert_close(got, want, 2e-4);
        }
    }
}

#[test]
fn display_p3_colours_outside_srgb_grade_at_the_srgb_boundary() {
    let lut = parse(&cube_source("", 17, |c| c));
    let encode = |c: [f32; 3]| c.map(|v| srgb_oetf_scalar(v.clamp(0.0, 1.0)));
    let red = map_to_gamut(
        display_p3_to_srgb_lin([1.0, 0.0, 0.0]),
        OutputColorSpace::SRgb,
    );
    let got = lut.apply([1.0, 0.0, 0.0], 1.0, OutputColorSpace::DisplayP3);
    assert_close(got, encode(srgb_lin_to_display_p3(red)), 2e-4);
    assert!(got[0] < 0.99, "{got:?}");
}
