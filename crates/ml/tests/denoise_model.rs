use std::path::{Path, PathBuf};
use std::time::Instant;

use ml::runtime::SessionConfig;
use ml::{Denoiser, RuntimeMode};

fn fixture(name: &str) -> Option<PathBuf> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    path.exists().then_some(path)
}

fn clean_tile(edge: usize) -> Vec<f32> {
    let mut chw = vec![0.0f32; 3 * edge * edge];
    chw.chunks_exact_mut(edge * edge)
        .zip([1.0, 0.9, 0.75])
        .for_each(|(channel, tint)| {
            channel
                .chunks_exact_mut(edge)
                .enumerate()
                .for_each(|(y, row)| {
                    let band = if (y / 64) % 2 == 0 { 0.0 } else { 0.08 };
                    row.iter_mut().enumerate().for_each(|(x, v)| {
                        let ramp = 0.2 + 0.6 * x as f32 / edge as f32;
                        *v = (ramp * tint + band).clamp(0.0, 1.0);
                    });
                });
        });
    chw
}

fn noisy(clean: &[f32], sigma: f32) -> Vec<f32> {
    let mut seed: u32 = 0x9e37_79b9;
    let mut uniform = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed as f32 / u32::MAX as f32
    };
    clean
        .iter()
        .map(|v| {
            let gaussian = (0..4).map(|_| uniform()).sum::<f32>() - 2.0;
            (v + gaussian * sigma * 1.732).clamp(0.0, 1.0)
        })
        .collect()
}

fn mean(values: &[f32]) -> f32 {
    values.iter().sum::<f32>() / values.len() as f32
}

fn rms_error(a: &[f32], b: &[f32]) -> f32 {
    let sum: f32 = a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum();
    (sum / a.len() as f32).sqrt()
}

#[test]
fn nind_removes_noise_without_shifting_brightness() {
    let Some(path) = fixture("nind_denoise_utnet_684.onnx") else {
        eprintln!("skipping nind: fixture missing");
        return;
    };
    let mode = match std::env::var("DENOISE_RUNTIME").as_deref() {
        Ok("gpu") => RuntimeMode::Gpu,
        _ => RuntimeMode::Cpu,
    };
    let mut denoiser = Denoiser::open(&path, mode, &SessionConfig::default()).unwrap();
    let edge = denoiser.tile_edge();
    assert_eq!(edge, 504);
    let clean = clean_tile(edge);
    let input = noisy(&clean, 0.05);

    let started = Instant::now();
    let output = denoiser.run(&input).unwrap();
    let elapsed = started.elapsed();

    let noise_in = rms_error(&input, &clean);
    let noise_out = rms_error(&output, &clean);
    let drift = mean(&output) - mean(&input);
    eprintln!(
        "nind {} {:?}: noise {noise_in:.4} -> {noise_out:.4}, mean drift {drift:+.5}, range {:.3}..{:.3}",
        denoiser.backend().as_str(),
        elapsed,
        output.iter().copied().fold(f32::INFINITY, f32::min),
        output.iter().copied().fold(f32::NEG_INFINITY, f32::max),
    );
    assert_eq!(output.len(), input.len());
    assert!(noise_out < noise_in * 0.5, "{noise_in} -> {noise_out}");
    assert!(drift.abs() < 0.01, "mean drift {drift}");
}
