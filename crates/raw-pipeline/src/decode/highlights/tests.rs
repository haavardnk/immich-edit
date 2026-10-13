use super::{reconstruct_mosaic, reconstruct_rgb};
use crate::sensor_sample::cfa_block;

const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";
const WB: [f32; 3] = [2.0, 1.0, 1.5];
const WB_RAW: [f32; 4] = [2.0, 1.0, 1.5, 0.0];
const SCENE: [f32; 3] = [1.0, 0.9, 0.7];
const WIDTH: usize = 96;
const HEIGHT: usize = 64;

fn exposure(x: usize, peak: f32) -> f32 {
    0.3 + (peak - 0.3) * x as f32 / (WIDTH - 1) as f32
}

fn scene_raw(x: usize, c: usize, scene: [f32; 3], peak: f32) -> f32 {
    exposure(x, peak) * scene[c] / WB[c]
}

fn mosaic(pattern: &str, scene: [f32; 3], peak: f32, saturation: f32) -> (Vec<f32>, Vec<f32>) {
    let (dim, table) = cfa_block(pattern);
    let truth: Vec<f32> = (0..WIDTH * HEIGHT)
        .map(|i| {
            scene_raw(
                i % WIDTH,
                table[(i / WIDTH % dim) * dim + i % WIDTH % dim],
                scene,
                peak,
            )
        })
        .collect();
    let clipped = truth.iter().map(|v| v.min(saturation)).collect();
    (truth, clipped)
}

fn color_at(pattern: &str, i: usize) -> usize {
    let (dim, table) = cfa_block(pattern);
    table[(i / WIDTH % dim) * dim + i % WIDTH % dim]
}

#[test]
fn mosaic_recovers_clipped_green() {
    for (pattern, saturation) in ["RGGB", "GBRG", XTRANS]
        .into_iter()
        .flat_map(|p| [(p, 1.0), (p, 1.069)])
    {
        let (truth, clipped) = mosaic(pattern, SCENE, 1.95, saturation);
        let mut out = clipped.clone();
        reconstruct_mosaic(&mut out, WIDTH, HEIGHT, pattern, WB_RAW);
        for i in 0..WIDTH * HEIGHT {
            if clipped[i] < 0.98 * saturation && out[i] != clipped[i] {
                panic!(
                    "{pattern}@{saturation}: unclipped site {i} changed {} -> {}",
                    clipped[i], out[i]
                );
            }
            if out[i] < clipped[i] {
                panic!(
                    "{pattern}@{saturation}: site {i} decreased {} -> {}",
                    clipped[i], out[i]
                );
            }
            let k = exposure(i % WIDTH, 1.95);
            if color_at(pattern, i) != 1 || !(1.25 * saturation..=1.9).contains(&k) {
                continue;
            }
            let err = (out[i] - truth[i]).abs() / truth[i];
            if err > 0.08 {
                panic!(
                    "{pattern}@{saturation}: green at k={k} got {} want {}",
                    out[i], truth[i]
                );
            }
        }
    }
}

#[test]
fn white_with_two_clipped_colors_stays_neutral() {
    let (truth, clipped) = mosaic("RGGB", [1.0; 3], 2.4, 1.0);
    let mut out = clipped;
    reconstruct_mosaic(&mut out, WIDTH, HEIGHT, "RGGB", WB_RAW);
    for i in (0..WIDTH * HEIGHT).filter(|i| (1.6..=1.9).contains(&exposure(i % WIDTH, 2.4))) {
        let c = color_at("RGGB", i);
        let err = (out[i] - truth[i]).abs() / truth[i];
        if err > 0.06 {
            panic!(
                "channel {c} at site {i} got {} want {}",
                out[i] * WB[c],
                truth[i] * WB[c]
            );
        }
    }
}

#[test]
fn unclipped_mosaic_is_untouched() {
    let (truth, _) = mosaic("RGGB", SCENE, 1.0, 1.0);
    let mut out = truth.clone();
    reconstruct_mosaic(&mut out, WIDTH, HEIGHT, "RGGB", WB_RAW);
    if out != truth {
        panic!("unclipped mosaic changed");
    }
}

#[test]
fn rgb_recovers_clipped_green() {
    let truth: Vec<f32> = (0..WIDTH * HEIGHT * 3)
        .map(|i| scene_raw(i / 3 % WIDTH, i % 3, SCENE, 1.95))
        .collect();
    let mut out: Vec<f32> = truth.iter().map(|v| v.min(1.0)).collect();
    reconstruct_rgb(&mut out, WIDTH, HEIGHT, WB_RAW);
    for i in (1..out.len()).step_by(3) {
        let k = exposure(i / 3 % WIDTH, 1.95);
        if !(1.25..=1.9).contains(&k) {
            continue;
        }
        let err = (out[i] - truth[i]).abs() / truth[i];
        if err > 0.08 {
            panic!("green at k={k} got {} want {}", out[i], truth[i]);
        }
    }
}

#[test]
fn two_clipped_band_keeps_surrounding_hue() {
    let sky = [0.8, 0.9, 1.0];
    for pattern in ["RGGB", XTRANS] {
        let (_, clipped) = mosaic(pattern, sky, 2.4, 1.0);
        let mut out = clipped;
        reconstruct_mosaic(&mut out, WIDTH, HEIGHT, pattern, WB_RAW);
        let mut sum = [0.0f32; 3];
        let mut count = [0.0f32; 3];
        for i in (0..WIDTH * HEIGHT).filter(|i| (1.7..=2.3).contains(&exposure(i % WIDTH, 2.4))) {
            let c = color_at(pattern, i);
            sum[c] += out[i] * WB[c] / exposure(i % WIDTH, 2.4);
            count[c] += 1.0;
        }
        let mean: [f32; 3] = std::array::from_fn(|c| sum[c] / count[c]);
        for c in 1..3 {
            let ratio = mean[c] / mean[0];
            let want = sky[c] / sky[0];
            if (ratio - want).abs() > 0.05 * want {
                panic!("{pattern}: channel {c} ratio {ratio} want {want}");
            }
        }
    }
}

#[test]
fn bright_core_on_darker_surface_turns_neutral() {
    let wall = [1.8, 1.08, 0.63];
    let core = [3.0; 3];
    for pattern in ["RGGB", XTRANS] {
        let mut out: Vec<f32> = (0..WIDTH * HEIGHT)
            .map(|i| {
                let c = color_at(pattern, i);
                let scene = if i % WIDTH < WIDTH / 2 { wall } else { core };
                (scene[c] / WB[c]).min(1.0)
            })
            .collect();
        reconstruct_mosaic(&mut out, WIDTH, HEIGHT, pattern, WB_RAW);
        let mut sum = [0.0f32; 3];
        let mut count = [0.0f32; 3];
        for i in (0..WIDTH * HEIGHT).filter(|i| i % WIDTH >= WIDTH / 2 + 16) {
            let c = color_at(pattern, i);
            sum[c] += out[i] * WB[c];
            count[c] += 1.0;
        }
        let mean: [f32; 3] = std::array::from_fn(|c| sum[c] / count[c]);
        for c in 1..3 {
            let ratio = mean[c] / mean[0];
            if (ratio - 1.0).abs() > 0.05 {
                panic!("{pattern}: channel {c} ratio {ratio} want 1");
            }
        }
    }
}

fn surfaces(pattern: &str, scenes: [[f32; 3]; 3]) -> Vec<f32> {
    let mut out: Vec<f32> = (0..WIDTH * HEIGHT)
        .map(|i| {
            let c = color_at(pattern, i);
            let texture = 1.0 - 0.1 * ((i * 7919) % 1000) as f32 / 1000.0;
            (texture * scenes[i % WIDTH * 3 / WIDTH][c] / WB[c]).min(1.0)
        })
        .collect();
    reconstruct_mosaic(&mut out, WIDTH, HEIGHT, pattern, WB_RAW);
    out
}

fn mean_level(pattern: &str, out: &[f32], columns: std::ops::Range<usize>) -> [f32; 3] {
    let mut sum = [0.0f32; 3];
    let mut count = [0.0f32; 3];
    for i in (0..WIDTH * HEIGHT).filter(|i| columns.contains(&(i % WIDTH))) {
        let c = color_at(pattern, i);
        sum[c] += out[i] * WB[c];
        count[c] += 1.0;
    }
    std::array::from_fn(|c| sum[c] / count[c])
}

#[test]
fn two_clipped_white_beside_color_gets_no_tint() {
    let wall = [1.9, 1.5, 1.0];
    let white = [1.7; 3];
    for pattern in ["RGGB", XTRANS] {
        let out = surfaces(pattern, [wall, white, white]);
        let mean = mean_level(pattern, &out, 48..WIDTH);
        if mean[1] > mean[0] || mean[2] > mean[0] {
            panic!("{pattern}: white turned {mean:?}");
        }
    }
}

#[test]
fn small_clipped_region_gets_one_hue() {
    let left = [1.9, 1.5, 1.0];
    let right = [1.5, 1.5, 1.4];
    for pattern in ["RGGB", XTRANS] {
        let out = surfaces(pattern, [left, [3.0; 3], right]);
        let near_left = mean_level(pattern, &out, 36..42);
        let near_right = mean_level(pattern, &out, 54..60);
        for c in 1..3 {
            let a = near_left[c] / near_left[0];
            let b = near_right[c] / near_right[0];
            if (a - b).abs() > 0.02 {
                panic!("{pattern}: channel {c} ratio {a} beside left, {b} beside right");
            }
        }
    }
}

#[test]
fn blurred_white_core_on_colored_wall_stays_white() {
    let wall = [1.45, 1.12, 0.85];
    let core = [2.4; 3];
    for pattern in ["RGGB", XTRANS] {
        let mut out: Vec<f32> = (0..WIDTH * HEIGHT)
            .map(|i| {
                let c = color_at(pattern, i);
                let t = ((i % WIDTH) as f32 - 32.0).clamp(0.0, 32.0) / 32.0;
                ((wall[c] + (core[c] - wall[c]) * t) / WB[c]).min(1.0)
            })
            .collect();
        reconstruct_mosaic(&mut out, WIDTH, HEIGHT, pattern, WB_RAW);
        let mean = mean_level(pattern, &out, 72..WIDTH);
        let ratios = mean.map(|m| m / mean[0]);
        if ratios.iter().any(|r| *r < 0.9) {
            panic!("{pattern}: ratios {ratios:?} want white");
        }
    }
}

#[test]
fn fully_clipped_mosaic_turns_neutral() {
    let mut out = vec![1.0f32; WIDTH * HEIGHT];
    reconstruct_mosaic(&mut out, WIDTH, HEIGHT, "RGGB", WB_RAW);
    for (i, v) in out.iter().enumerate() {
        let c = color_at("RGGB", i);
        if (v * WB[c] - 2.0).abs() > 1e-4 {
            panic!(
                "site {i} channel {c} scaled to {}, want neutral 2.0",
                v * WB[c]
            );
        }
    }
}
