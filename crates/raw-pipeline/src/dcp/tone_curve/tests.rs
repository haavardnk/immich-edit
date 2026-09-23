use super::*;

fn reference_eval(curve: &[[f32; 2]], x: f32) -> f32 {
    if curve.len() < 2 {
        return x;
    }
    let x = x.clamp(0.0, 1.0);
    if x <= curve[0][0] {
        return curve[0][1];
    }
    if x >= curve[curve.len() - 1][0] {
        return curve[curve.len() - 1][1];
    }
    let hi = curve.partition_point(|p| p[0] < x).max(1);
    let a = curve[hi - 1];
    let b = curve[hi];
    let span = b[0] - a[0];
    if span <= 1e-9 {
        return a[1];
    }
    let t = (x - a[0]) / span;
    a[1] + (b[1] - a[1]) * t
}

fn sampled(n: usize, warp: impl Fn(f32) -> f32) -> Vec<[f32; 2]> {
    (0..n)
        .map(|i| {
            let x = warp(i as f32 / (n - 1) as f32);
            [x, x.sqrt()]
        })
        .collect()
}

#[test]
fn bucketed_eval_matches_a_full_binary_search_bit_for_bit() {
    let curves = [
        crate::color::DCP_FALLBACK_TONE_CURVE.to_vec(),
        sampled(8192, |x| x),
        sampled(4096, |x| x.powi(4)),
        sampled(300, |x| 0.2 + 0.6 * x),
        vec![[0.0, 0.0], [0.5, 0.2], [0.5, 0.7], [0.5, 0.8], [1.0, 1.0]],
        vec![[0.0, 0.1], [1.0, 0.9]],
        vec![[0.3, 0.3]],
    ];
    let edges = (0..=BUCKETS).flat_map(|b| {
        let e = b as f32 / BUCKETS as f32;
        [e.next_down(), e, e.next_up()]
    });
    let sweep = (0..=100_000).map(|i| i as f32 / 100_000.0);
    let probes: Vec<f32> = edges
        .chain(sweep)
        .chain([-1.0, 2.0, f32::NAN, f32::INFINITY, f32::MIN_POSITIVE])
        .collect();
    for points in curves {
        let curve = ToneCurve::new(points.clone());
        for &x in &probes {
            let got = curve.eval(x);
            let want = reference_eval(&points, x);
            if got.to_bits() != want.to_bits() {
                panic!("{} point curve at {x:e}: {got:e} vs {want:e}", points.len());
            }
        }
    }
}
