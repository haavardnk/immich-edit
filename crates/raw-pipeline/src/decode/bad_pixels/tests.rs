use super::*;

const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

fn hash(i: usize) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

fn textured(w: usize, h: usize) -> Vec<f32> {
    (0..w * h)
        .map(|i| {
            let (x, y) = ((i % w) as f32, (i / w) as f32);
            0.2 + 0.1 * (x * 0.07).sin() * (y * 0.05).cos() + 0.01 * hash(i)
        })
        .collect()
}

fn star(data: &mut [f32], w: usize, center: [f32; 2], sigma: f32, peak: f32) {
    data.iter_mut().enumerate().for_each(|(i, v)| {
        let dx = (i % w) as f32 - center[0];
        let dy = (i / w) as f32 - center[1];
        *v += peak * (-(dx * dx + dy * dy) / (2.0 * sigma * sigma)).exp();
    });
}

fn repaired(
    mut data: Vec<f32>,
    w: usize,
    h: usize,
    pattern: &str,
    flagged: &[[usize; 2]],
) -> Vec<f32> {
    repair(&mut Mosaic::whole(&mut data, w, h, pattern), flagged);
    data
}

fn changed(before: &[f32], after: &[f32]) -> Vec<usize> {
    before
        .iter()
        .zip(after)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn hot_and_dead_sites_are_replaced_by_their_colour_ring() {
    let (w, h) = (32, 24);
    for pattern in ["RGGB", "GBRG", XTRANS] {
        let clean = textured(w, h);
        let mut data = clean.clone();
        let hot = 10 * w + 11;
        let dead = 15 * w + 20;
        let corner = 0;
        data[hot] = 0.9;
        data[dead] = 0.0;
        data[corner] = 1.0;
        let out = repaired(data.clone(), w, h, pattern, &[]);
        assert_eq!(changed(&data, &out), vec![corner, hot, dead], "{pattern}");
        for i in [hot, dead, corner] {
            assert!(
                (out[i] - clean[i]).abs() < 0.05,
                "{pattern} site {i}: {}",
                out[i]
            );
        }
    }
}

#[test]
fn point_lights_survive() {
    let (w, h) = (40, 40);
    for pattern in ["RGGB", XTRANS] {
        for sigma in [0.6, 0.8, 1.0] {
            for center in [
                [20.0, 20.0],
                [21.0, 20.0],
                [20.0, 21.0],
                [21.0, 21.0],
                [20.5, 20.0],
                [20.0, 20.5],
                [20.5, 20.5],
                [20.3, 20.7],
            ] {
                let mut data = textured(w, h);
                star(&mut data, w, center, sigma, 0.7);
                let out = repaired(data.clone(), w, h, pattern, &[]);
                assert!(
                    changed(&data, &out).is_empty(),
                    "{pattern} sigma {sigma} at {center:?} was treated as a hot pixel"
                );
            }
        }
    }
}

#[test]
fn texture_and_noise_are_left_alone() {
    let (w, h) = (64, 48);
    for pattern in ["RGGB", "BGGR", XTRANS] {
        let data: Vec<f32> = textured(w, h)
            .iter()
            .enumerate()
            .map(|(i, v)| v * (0.9 + 0.2 * hash(i + 7)))
            .collect();
        let out = repaired(data.clone(), w, h, pattern, &[]);
        assert!(changed(&data, &out).is_empty(), "{pattern}");
    }
}

#[test]
fn flagged_sites_are_replaced_from_unflagged_neighbours() {
    let (w, h) = (16, 16);
    let mut data = vec![0.5f32; w * h];
    data[6 * w + 6] = 0.52;
    data[6 * w + 8] = 0.9;
    data[8 * w + 6] = 0.1;
    let flagged = [[6, 6], [8, 6], [6, 8], [40, 2]];
    let out = repaired(data, w, h, "RGGB", &flagged);
    for [x, y] in &flagged[..3] {
        assert_eq!(out[y * w + x], 0.5, "({x}, {y})");
    }
}

#[test]
fn isolated_sites_without_enough_neighbours_are_kept() {
    let mut data = vec![0.1f32, 0.1, 0.9, 0.1];
    repair(&mut Mosaic::whole(&mut data, 4, 1, "RGGB"), &[]);
    assert_eq!(data[2], 0.9);
}
