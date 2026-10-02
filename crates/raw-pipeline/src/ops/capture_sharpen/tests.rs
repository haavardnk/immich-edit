use super::psf::pixel_level;
use super::*;
use crate::frame::PreviewMode;
use crate::ops::{OpScratch, RenderContext};

fn ctx(is_raw: bool, capture_sigma: Option<CaptureSigma>) -> OpContext {
    OpContext {
        render: RenderContext {
            wb_coeffs: [1.0; 4],
            cam_to_srgb: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            is_raw,
            capture_sigma,
            preview_mode: PreviewMode::None,
            roi: None,
            white: crate::color::SceneWhite::Display,
            dcp: None,
            output_scale: 1.0,
        },
        scratch: OpScratch::default(),
    }
}

fn detail_scene(w: usize, h: usize) -> Vec<f32> {
    let mut rgb = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let bar = if (x / 5 + y / 7) % 2 == 0 {
                0.55f32
            } else {
                0.18
            };
            let spot = if (x % 23 == 11) && (y % 19 == 9) {
                0.25
            } else {
                0.0
            };
            let v = (bar + spot).clamp(0.02, 0.9);
            let i = (y * w + x) * 3;
            rgb[i] = v;
            rgb[i + 1] = v * 0.96;
            rgb[i + 2] = v * 0.92;
        }
    }
    rgb
}

fn gray(w: usize, h: usize, mut value: impl FnMut(usize, usize) -> f32) -> Vec<f32> {
    (0..w * h).flat_map(|i| [value(i % w, i / w); 3]).collect()
}

fn blur_rgb(rgb: &[f32], w: usize, h: usize, sigma: CaptureSigma) -> Vec<f32> {
    let psf = Psf::new(sigma, w, h);
    let mut out = rgb.to_vec();
    for c in 0..3 {
        let plane: Vec<f32> = (0..w * h).map(|i| rgb[i * 3 + c]).collect();
        let mut dst = vec![0.0f32; w * h];
        convolve_with(&plane, &mut dst, w, h, &psf, |_, acc, row| {
            row.copy_from_slice(acc)
        });
        for i in 0..w * h {
            out[i * 3 + c] = dst[i];
        }
    }
    out
}

fn rmse(a: &[f32], b: &[f32]) -> f32 {
    let sum: f32 = a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum();
    (sum / a.len() as f32).sqrt()
}

#[test]
fn banded_convolution_matches_direct_blur() {
    let w = 37;
    let h = 64 * 2 + 13;
    let src: Vec<f32> = (0..w * h)
        .map(|i| ((i * 7919) % 101) as f32 / 101.0)
        .collect();
    let half_diag = half_diagonal(w, h);
    for sigma in [
        CaptureSigma::uniform(1.3),
        CaptureSigma {
            centre: 0.4,
            corner: 1.9,
        },
    ] {
        let kernels = sigma.kernels();
        let kernel_at = |x: usize, y: usize| {
            &kernels[usize::from(pixel_level(x, y, w, h, half_diag, kernels.len()))]
        };
        let mut banded = vec![0.0f32; w * h];
        convolve_with(
            &src,
            &mut banded,
            w,
            h,
            &Psf::new(sigma, w, h),
            |_, acc, row| row.copy_from_slice(acc),
        );
        let clamp = |v: usize, r: usize, n: usize| v.saturating_sub(r).min(n - 1);
        let worst = (0..w * h)
            .map(|p| {
                let (x, y) = (p % w, p / w);
                let ky = kernel_at(x, y);
                let direct: f32 = ky
                    .iter()
                    .enumerate()
                    .map(|(j, kj)| {
                        let sy = clamp(y + j, ky.len() / 2, h);
                        let kx = kernel_at(x, sy);
                        kj * kx
                            .iter()
                            .enumerate()
                            .map(|(i, ki)| ki * src[sy * w + clamp(x + i, kx.len() / 2, w)])
                            .sum::<f32>()
                    })
                    .sum();
                (direct - banded[p]).abs()
            })
            .fold(0.0f32, f32::max);
        if worst > 1e-5 {
            panic!("{sigma:?}: banded blur drifted {worst} from the direct blur");
        }
    }
}

#[test]
fn local_extreme_uses_each_pixel_radius() {
    let w = 41;
    let h = 64 + 29;
    let src: Vec<f32> = (0..w * h)
        .map(|i| ((i * 104_729) % 97) as f32 / 97.0)
        .collect();
    let sigma = CaptureSigma {
        centre: 0.3,
        corner: 2.0,
    };
    let kernels = sigma.kernels();
    let half_diag = half_diagonal(w, h);
    let mut lo = vec![0.0f32; w * h];
    local_extreme(&src, &mut lo, w, h, &Psf::new(sigma, w, h), f32::min);
    for (p, got) in lo.iter().enumerate() {
        let (x, y) = (p % w, p / w);
        let r = kernels[usize::from(pixel_level(x, y, w, h, half_diag, kernels.len()))].len() / 2;
        let want = (y.saturating_sub(r)..=(y + r).min(h - 1))
            .flat_map(|sy| (x.saturating_sub(r)..=(x + r).min(w - 1)).map(move |sx| (sx, sy)))
            .map(|(sx, sy)| src[sy * w + sx])
            .fold(f32::INFINITY, f32::min);
        if *got != want {
            panic!("min at ({x}, {y}) radius {r}: got {got}, want {want}");
        }
    }
}

#[test]
fn recovers_detail_lost_to_sensor_blur() {
    let w = 96;
    let h = 96;
    let sharp = detail_scene(w, h);
    for sigma in [0.5f32, 0.8, 1.2].map(CaptureSigma::uniform) {
        let blurred = blur_rgb(&sharp, w, h, sigma);
        let mut image = LinearImage::new(blurred.clone(), w, h);
        apply_capture_sharpen(&mut image, sigma);
        let before = rmse(&blurred, &sharp);
        let after = rmse(&image.rgb, &sharp);
        eprintln!("{sigma:?}: rmse {before} -> {after}");
        if after >= before {
            panic!("{sigma:?}: sharpening did not reduce error ({before} -> {after})");
        }
    }
}

#[test]
fn corner_boost_recovers_soft_corners() {
    let w = 160;
    let h = 120;
    let sharp = detail_scene(w, h);
    let lens = CaptureSigma {
        centre: 0.6,
        corner: 1.5,
    };
    let blurred = blur_rgb(&sharp, w, h, lens);
    let corners: Vec<usize> = (0..w * h)
        .filter(|&p| (p % w).abs_diff(w / 2) > w * 3 / 8 && (p / w).abs_diff(h / 2) > h * 3 / 8)
        .flat_map(|p| [p * 3, p * 3 + 1, p * 3 + 2])
        .collect();
    let corner_rmse = |rgb: &[f32]| {
        let a: Vec<f32> = corners.iter().map(|&i| rgb[i]).collect();
        let b: Vec<f32> = corners.iter().map(|&i| sharp[i]).collect();
        rmse(&a, &b)
    };
    let mut flat = LinearImage::new(blurred.clone(), w, h);
    apply_capture_sharpen(&mut flat, CaptureSigma::uniform(lens.centre));
    let mut boosted = LinearImage::new(blurred.clone(), w, h);
    apply_capture_sharpen(&mut boosted, lens);
    let before = corner_rmse(&blurred);
    let uniform = corner_rmse(&flat.rgb);
    let graded = corner_rmse(&boosted.rgb);
    eprintln!("corner rmse {before} -> uniform {uniform}, boosted {graded}");
    if graded >= uniform * 0.9 {
        panic!("corner boost did not sharpen soft corners: uniform {uniform}, boosted {graded}");
    }
}

#[test]
fn contrast_gate_follows_lightness() {
    let w = 16;
    let h = 16;
    for (low, high, opens) in [(0.020f32, 0.026f32, true), (0.70, 0.72, false)] {
        let rgb = gray(w, h, |x, _| if x < w / 2 { low } else { high });
        let image = LinearImage::new(rgb, w, h);
        let lum: Vec<f32> = (0..w * h).map(|i| image.rgb[i * 3]).collect();
        let light: Vec<f32> = lum.iter().map(|v| v.cbrt()).collect();
        let blend = build_blend(&image, &lum, &light, w, h);
        let edge = blend[8 * w + w / 2];
        eprintln!("step {low} -> {high}: blend {edge}");
        if (edge > 0.1) != opens {
            panic!("step {low} -> {high}: blend {edge}, expected open = {opens}");
        }
    }
}

#[test]
fn leaves_flat_noise_alone() {
    let w = 64;
    let h = 64;
    let mut seed: u32 = 0x9e37_79b9;
    let rgb = gray(w, h, |_, _| {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        0.35 + (seed as f32 / u32::MAX as f32 - 0.5) * 0.01
    });
    let mut image = LinearImage::new(rgb.clone(), w, h);
    apply_capture_sharpen(&mut image, CaptureSigma::uniform(0.8));
    let delta = rmse(&image.rgb, &rgb);
    eprintln!("flat noise delta {delta}");
    if delta > 1e-4 {
        panic!("flat noise was amplified by {delta}");
    }
}

#[test]
fn does_not_overshoot_clipped_highlights() {
    let w = 64;
    let h = 64;
    let rgb = gray(w, h, |x, _| if x < w / 2 { 0.05 } else { 1.0 });
    let blurred = blur_rgb(&rgb, w, h, CaptureSigma::uniform(1.0));
    let peak_before = blurred.iter().copied().fold(0.0f32, f32::max);
    let mut image = LinearImage::new(blurred, w, h);
    apply_capture_sharpen(&mut image, CaptureSigma::uniform(1.0));
    let peak_after = image.rgb.iter().copied().fold(0.0f32, f32::max);
    eprintln!("clip peak {peak_before} -> {peak_after}");
    if peak_after > peak_before + 1e-4 {
        panic!("clipped edge overshot from {peak_before} to {peak_after}");
    }
}

#[test]
fn skipped_without_raw_or_sigma() {
    let edits = Edits::default();
    assert!(edits.detail.capture_sharpen);
    let sigma = Some(CaptureSigma::uniform(0.7));
    assert!(context_sigma(&ctx(true, sigma), &edits).is_some());
    assert!(context_sigma(&ctx(false, sigma), &edits).is_none());
    assert!(context_sigma(&ctx(true, None), &edits).is_none());
    assert!(context_sigma(&ctx(true, Some(CaptureSigma::uniform(0.2))), &edits).is_none());
    let mut off = Edits::default();
    off.detail.capture_sharpen = false;
    assert!(context_sigma(&ctx(true, sigma), &off).is_none());
}

#[test]
fn corner_boost_scales_with_the_grid() {
    let mut edits = Edits::default();
    let cases = [
        (0.0, 0.8, 1.0, Some((0.8, 0.8))),
        (50.0, 0.8, 2.0, Some((0.4, 0.65))),
        (100.0, 1.5, 1.0, Some((1.5, 2.0))),
        (0.0, 0.2, 1.0, None),
        (40.0, 0.2, 1.0, Some((0.2, 0.6))),
    ];
    for (boost, measured, block, want) in cases {
        edits.detail.capture_corner_boost = boost;
        let got = CaptureSigma::on_grid(measured, &edits, block)
            .active()
            .map(|s| (s.centre, s.corner));
        let close = match (got, want) {
            (Some(a), Some(b)) => (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6,
            (a, b) => a.is_none() && b.is_none(),
        };
        if !close {
            panic!("boost {boost} sigma {measured} block {block}: got {got:?}, want {want:?}");
        }
    }
    edits.detail.capture_corner_boost = 0.0;
    assert_eq!(
        CaptureSigma::on_grid(0.8, &edits, 1.0).kernels(),
        vec![gaussian_kernel(0.8)]
    );
}

#[test]
fn manifest_round_trip_records_off_state_and_boost() {
    let op = CaptureSharpenOp;
    let mut edits = Edits::default();
    assert!(op.to_doc(&edits).is_none());
    for (enabled, boost) in [(false, 0.0), (true, 35.0), (false, 60.0)] {
        edits.detail.capture_sharpen = enabled;
        edits.detail.capture_corner_boost = boost;
        let doc = op.to_doc(&edits).expect("doc");
        let mut restored = Edits::default();
        op.apply_doc(&doc, &mut restored);
        assert_eq!(restored.detail.capture_sharpen, enabled);
        assert_eq!(restored.detail.capture_corner_boost, boost);
    }
}
