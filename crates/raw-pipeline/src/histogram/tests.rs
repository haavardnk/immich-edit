use super::*;

#[test]
fn display_luma_cases() {
    let cases = [
        ((0, 0, 0), 0),
        ((255, 255, 255), 255),
        ((255, 0, 0), 54),
        ((0, 255, 0), 182),
        ((0, 0, 255), 18),
        ((200, 100, 20), 115),
    ];
    for ((r, g, b), expected) in cases {
        let level = display_luma(r, g, b);
        if level != expected {
            panic!("({r}, {g}, {b}) -> {level}, want {expected}");
        }
    }
}

#[test]
fn display_luma_keeps_every_grey_on_its_own_level() {
    let misplaced: Vec<u8> = (0..=255u8)
        .filter(|&k| display_luma(k, k, k) != k as usize)
        .collect();
    if !misplaced.is_empty() {
        panic!("greys off their level: {misplaced:?}");
    }
}

#[test]
fn from_rgb_u8_sampling_cases() {
    let cases = [
        (100, 100, 10_000),
        (1000, 500, 500_000),
        (1001, 500, 250_250),
    ];
    for (width, height, expected) in cases {
        let pixels = vec![128u8; width * height * 3];
        let counted = Histogram::from_rgb_u8(&pixels, width, height).pixel_count();
        if counted != expected {
            panic!("{width}x{height}: counted {counted}, want {expected}");
        }
    }
}

#[test]
fn from_counts_splits_channels_in_order() {
    let counts: Vec<u32> = (0..4 * BINS as u32).collect();
    let histogram = Histogram::from_counts(&counts);
    let firsts = [
        histogram.r[0],
        histogram.g[0],
        histogram.b[0],
        histogram.l[0],
    ];
    if firsts != [0, 256, 512, 768] {
        panic!("channel starts {firsts:?}");
    }
}
