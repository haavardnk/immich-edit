use super::chroma::{self, from_chroma, to_chroma};
use super::estimate::{self, NoiseCurve};
use super::shrink::{LevelParams, gain};
use super::{atrous, luma};
use crate::math;
use crate::ops::LinearImage;

fn gaussian(seed: u32) -> impl FnMut() -> f32 {
    let mut seed = seed;
    move || {
        let mut s = 0.0f32;
        for _ in 0..12 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            s += seed as f32 / u32::MAX as f32;
        }
        s - 6.0
    }
}

fn noisy_image(w: usize, h: usize, sigma: f32) -> LinearImage {
    let mut gauss = gaussian(0x1234_5678);
    let rgb = (0..w * h * 3)
        .map(|i| {
            let x = (i / 3) % w;
            let base = if x < w / 2 { 0.2 } else { 0.6 };
            let tint = [1.0, 0.8, 0.5][i % 3];
            (base * tint + gauss() * sigma).max(0.0)
        })
        .collect();
    LinearImage::new(rgb, w, h)
}

#[test]
fn atrous_matches_clamped_reference() {
    for (w, h, step) in [
        (1, 1, 1),
        (2, 3, 1),
        (3, 3, 2),
        (5, 4, 4),
        (9, 7, 2),
        (37, 29, 8),
    ] {
        let src: Vec<f32> = (0..w * h)
            .map(|i| ((i * 7919) % 97) as f32 / 97.0)
            .collect();
        let mut dst = vec![0.0; w * h];
        let mut tmp = vec![0.0; w * h];
        atrous::smooth(&src, &mut dst, &mut tmp, w, h, step);
        let at = |x: isize, y: isize| {
            src[y.clamp(0, h as isize - 1) as usize * w + x.clamp(0, w as isize - 1) as usize]
        };
        let taps =
            [-2isize, -1, 0, 1, 2].map(|k| (k * step as isize, atrous::B3[k.unsigned_abs()]));
        for (i, got) in dst.iter().enumerate() {
            let x = (i % w) as isize;
            let y = (i / w) as isize;
            let want: f32 = taps
                .iter()
                .map(|&(dy, ky)| {
                    ky * taps
                        .iter()
                        .map(|&(dx, kx)| kx * at(x + dx, y + dy))
                        .sum::<f32>()
                })
                .sum();
            assert!(
                (got - want).abs() < 1e-5,
                "{w}x{h} step {step} at {i}: {got} vs {want}"
            );
        }
    }
}

#[test]
fn gain_limits() {
    let p = LevelParams {
        lambda: 1.0,
        mu: 1.0,
        keep: 0.25,
    };
    for (energy, variance, want) in [
        (0.5, 0.0, 1.0),
        (0.0, 0.1, 0.25),
        (0.1, 0.1, 0.25),
        (1e6, 1.0, 1.0),
    ] {
        let g = gain(energy, variance, p);
        assert!(
            (g - want).abs() < 1e-3,
            "energy {energy} variance {variance}: {g}"
        );
    }
}

#[test]
fn chroma_round_trip() {
    for rgb in [
        [0.0, 0.0, 0.0],
        [1.0, 0.5, 0.25],
        [0.02, 0.9, 0.4],
        [3.0, 0.1, 0.0],
    ] {
        let [y, pb, pr] = to_chroma(rgb[0], rgb[1], rgb[2]);
        let back = from_chroma(y, pb, pr);
        for (a, b) in back.iter().zip(rgb) {
            assert!((a - b).abs() < 1e-5, "{rgb:?} -> {back:?}");
        }
    }
}

#[test]
fn chroma_denoise_preserves_pixel_luma() {
    let mut image = noisy_image(128, 96, 0.05);
    let before: Vec<f32> = image
        .rgb
        .chunks_exact(3)
        .map(|p| math::luma(p[0], p[1], p[2]))
        .collect();
    chroma::denoise(&mut image, chroma::level_params(100.0, 50.0, 50.0));
    for (i, (p, y)) in image.rgb.chunks_exact(3).zip(before).enumerate() {
        let after = math::luma(p[0], p[1], p[2]);
        assert!((after - y).abs() < 1e-5, "pixel {i}: {y} -> {after}");
    }
}

#[test]
fn luma_denoise_keeps_chroma_direction() {
    let mut image = noisy_image(128, 96, 0.05);
    let before = image.rgb.clone();
    luma::denoise(&mut image, luma::level_params(100.0, 50.0, 0.0));
    let offsets = |p: &[f32]| {
        let y = math::luma(p[0], p[1], p[2]);
        [p[0] - y, p[1] - y, p[2] - y]
    };
    let changed = image
        .rgb
        .iter()
        .zip(&before)
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    assert!(changed > image.rgb.len() / 2);
    for (a, b) in image.rgb.chunks_exact(3).zip(before.chunks_exact(3)) {
        let da = offsets(a);
        let db = offsets(b);
        assert!(
            (da[0] * db[1] - da[1] * db[0]).abs() < 1e-5,
            "{a:?} vs {b:?}"
        );
        assert!(
            (da[2] * db[1] - da[1] * db[2]).abs() < 1e-5,
            "{a:?} vs {b:?}"
        );
    }
}

#[test]
fn fit_caps_to_coarser_level() {
    let mut hist = vec![0u32; estimate::HIST_LEN];
    let m = estimate::mag_bin(0.1).unwrap_or_default();
    for i in 0..estimate::LUM_BINS {
        hist[i * estimate::MAG_BINS + m] = 1000;
    }
    let prev = NoiseCurve { a: 1e-4, b: 1e-3 };
    let capped = estimate::fit(&hist, Some(prev));
    assert!(capped.a <= prev.a * 0.3 + f32::EPSILON);
    assert!(capped.b <= prev.b * 0.3 + f32::EPSILON);
}

#[test]
fn fit_recovers_linear_noise_curve() {
    let curve = NoiseCurve { a: 1e-5, b: 4e-4 };
    let mut hist = vec![0u32; estimate::HIST_LEN];
    let mut gauss = gaussian(0x9E37_79B9);
    for i in 0..estimate::LUM_BINS {
        let y = ((i as f32 + 0.5) / estimate::LUM_BINS as f32).powi(2);
        let sigma = curve.variance(y).sqrt();
        for _ in 0..4000 {
            let d = gauss() * sigma;
            if let Some(m) = estimate::mag_bin(d.abs()) {
                hist[estimate::lum_bin(y) * estimate::MAG_BINS + m] += 1;
            }
        }
    }
    let fitted = estimate::fit(&hist, None);
    for y in [0.05f32, 0.3, 0.8] {
        let ratio = (fitted.variance(y) / curve.variance(y)).sqrt();
        assert!(
            (ratio - 1.0).abs() < 0.08,
            "y={y} ratio={ratio} fitted={fitted:?}"
        );
    }
}
