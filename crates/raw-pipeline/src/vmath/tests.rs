use super::*;

fn relative(a: f32, b: f32) -> f32 {
    (a - b).abs() / b.abs().max(f32::MIN_POSITIVE)
}

fn worst(
    samples: impl Iterator<Item = f32>,
    approx: impl Fn(f32) -> f32,
    exact: impl Fn(f32) -> f32,
    err: impl Fn(f32, f32) -> f32,
) -> (f32, f32) {
    samples
        .map(|x| (x, err(approx(x), exact(x))))
        .fold((0.0, 0.0), |w, s| if s.1 > w.1 { s } else { w })
}

fn log_spaced(lo: f32, hi: f32, n: usize) -> impl Iterator<Item = f32> {
    let a = lo.log2();
    let b = hi.log2();
    (0..=n).map(move |i| (a + (b - a) * i as f32 / n as f32).exp2())
}

fn linear(lo: f32, hi: f32, n: usize) -> impl Iterator<Item = f32> {
    (0..=n).map(move |i| lo + (hi - lo) * i as f32 / n as f32)
}

#[test]
fn accuracy_cases() {
    let cases: [(&str, (f32, f32), f32); 5] = [
        (
            "exp2",
            worst(linear(-126.0, 127.0, 200_000), exp2, f32::exp2, relative),
            4e-7,
        ),
        (
            "log2",
            worst(log_spaced(1e-30, 1e30, 200_000), log2, f32::log2, |a, b| {
                (a - b).abs() / b.abs().max(1.0)
            }),
            3e-7,
        ),
        (
            "log2 near 1",
            worst(linear(0.5, 2.0, 200_000), log2, f32::log2, |a, b| {
                (a - b).abs()
            }),
            3e-7,
        ),
        (
            "exp",
            worst(linear(-10.0, 10.0, 200_000), exp, f32::exp, relative),
            1e-6,
        ),
        (
            "tanh",
            worst(linear(-12.0, 12.0, 200_000), tanh, f32::tanh, |a, b| {
                (a - b).abs()
            }),
            3e-7,
        ),
    ];
    for (name, (x, err), bound) in cases {
        if err > bound {
            panic!("{name}: error {err:e} at {x} exceeds {bound:e}");
        }
    }
}

#[test]
fn pow_matches_powf_over_the_image_domain() {
    let exponents = [
        1.0 / 2.4,
        1.0 / 2.2,
        0.5,
        0.8,
        1.0,
        1.25,
        1.7,
        2.2,
        2.4,
        3.5,
    ];
    for y in exponents {
        let (x, err) = worst(
            log_spaced(1e-6, 64.0, 100_000),
            |x| pow(x, y),
            |x| x.powf(y),
            relative,
        );
        if err > 4e-6 {
            panic!("pow(x, {y}): error {err:e} at {x}");
        }
    }
}

#[test]
fn pow_edge_cases() {
    let cases = [
        (0.0, 2.2, 0.0),
        (0.0, 0.0, 1.0),
        (1.0, 2.2, 1.0),
        (4.0, 0.5, 2.0),
        (2.0, 0.0, 1.0),
    ];
    for (x, y, expected) in cases {
        let got = pow(x, y);
        if relative(got, expected) > 1e-6 && got != expected {
            panic!("pow({x}, {y}) = {got}, want {expected}");
        }
    }
    let nan_inputs = [(-1.0, 0.5), (f32::NAN, 2.0)];
    for (x, y) in nan_inputs {
        if !pow(x, y).is_nan() {
            panic!("pow({x}, {y}) should be NaN");
        }
    }
}

#[test]
fn exp2_saturates_instead_of_overflowing() {
    let cases = [
        (200.0, 2f32.powi(127)),
        (-200.0, 2f32.powi(-126)),
        (0.0, 1.0),
        (1.0, 2.0),
    ];
    for (x, expected) in cases {
        let got = exp2(x);
        if relative(got, expected) > 4e-7 {
            panic!("exp2({x}) = {got}, want {expected}");
        }
    }
}
