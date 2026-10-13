use super::apply_tone_regions_rgb;

#[test]
fn negative_highlights_compress_without_flattening() {
    for hl in [-1.0f32, -0.5] {
        let mut prev = 0.0f32;
        for step in 1..=800 {
            let x = step as f32 * 0.01;
            let (out, _, _) = apply_tone_regions_rgb(x, x * 0.5, x * 0.25, hl, 0.0);
            if out <= prev {
                panic!("hl {hl}: output not increasing at {x}: {out} <= {prev}");
            }
            if x > 0.5 && out >= x {
                panic!("hl {hl}: {x} was not compressed, got {out}");
            }
            prev = out;
        }
    }
}

#[test]
fn negative_highlights_keep_ratios_and_midtones() {
    let (r, g, b) = apply_tone_regions_rgb(2.0, 1.0, 0.5, -1.0, 0.0);
    if (g / r - 0.5).abs() > 1e-5 || (b / r - 0.25).abs() > 1e-5 {
        panic!("channel ratios changed: {r} {g} {b}");
    }
    let (m, _, _) = apply_tone_regions_rgb(0.18, 0.18, 0.18, -1.0, 0.0);
    if (m - 0.18).abs() > 1e-4 {
        panic!("mid grey moved to {m}");
    }
}
