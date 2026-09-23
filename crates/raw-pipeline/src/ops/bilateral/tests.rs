use super::*;

fn reference<const C: usize>(src: [&[f32]; C], w: usize, h: usize, p: Bilateral) -> [Vec<f32>; C] {
    let mut out: [Vec<f32>; C] = std::array::from_fn(|_| vec![0.0; w * h]);
    for (y, x) in (0..h).flat_map(|y| (0..w).map(move |x| (y, x))) {
        let y0 = y.saturating_sub(p.radius);
        let y1 = (y + p.radius).min(h - 1);
        let values = filter_edge_pixel(src, w, x, y, y0, y1, p);
        for c in 0..C {
            out[c][y * w + x] = values[c];
        }
    }
    out
}

fn plane(w: usize, h: usize, seed: u32) -> Vec<f32> {
    (0..w * h)
        .map(|i| {
            let v = (i as u32).wrapping_mul(2_654_435_761).wrapping_add(seed) >> 8;
            (v % 1000) as f32 / 1000.0
        })
        .collect()
}

fn params(radius: usize) -> Bilateral {
    Bilateral {
        radius,
        inv_2ss: 1.0 / (2.0 * (radius * radius) as f32),
        inv_2sr: 1.0 / (2.0 * 0.05 * 0.05),
    }
}

#[test]
fn row_kernel_matches_the_per_pixel_filter_bit_for_bit() {
    let cases = [(37, 23, 2), (64, 40, 4), (7, 9, 4), (3, 3, 2), (50, 5, 3)];
    for (w, h, radius) in cases {
        let a = plane(w, h, 1);
        let b = plane(w, h, 7);
        let p = params(radius);

        let mut one = vec![0.0; w * h];
        filter([&a], [&mut one], w, h, p);
        let [expected_one] = reference([&a], w, h, p);
        if one != expected_one {
            panic!("{w}x{h} r{radius}: one-channel filter differs from the reference");
        }

        let mut out_a = vec![0.0; w * h];
        let mut out_b = vec![0.0; w * h];
        filter([&a, &b], [&mut out_a, &mut out_b], w, h, p);
        let [expected_a, expected_b] = reference([&a, &b], w, h, p);
        if out_a != expected_a || out_b != expected_b {
            panic!("{w}x{h} r{radius}: two-channel filter differs from the reference");
        }
    }
}
