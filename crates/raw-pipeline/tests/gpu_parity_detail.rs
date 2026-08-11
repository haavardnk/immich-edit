mod common;

use common::{
    decode_jpeg_rgb, detail_frame, fine_texture_frame, haze_frame, mean_abs_delta,
    split_tone_frame, stripe_frame, synthetic_frame, try_renderer,
};
use raw_pipeline::edits::{ColorEdits, DcpEdits, DcpMode, DetailEdits, Edits, EffectsEdits};
use raw_pipeline::frame::RenderOptions;

#[test]
fn gpu_presence_sliders_match_cpu_via_fallback() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let frame = stripe_frame(48, 32);
    let mut edits = Edits::default();
    edits.basic.texture = 30.0;
    edits.basic.clarity = 20.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("presence mean abs delta = {delta:.3}");
    if delta > 8.0 {
        panic!("presence GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_dehaze_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let frame = haze_frame(96, 64);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("dehaze mean abs delta = {delta:.3}");
    if delta > 10.0 {
        panic!("dehaze GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_dehaze_with_presence_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let frame = haze_frame(96, 64);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;
    edits.basic.clarity = 80.0;
    edits.basic.texture = 80.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("dehaze+presence mean abs delta = {delta:.3}");
    if delta > 1.2 {
        panic!("dehaze+presence GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_shadows_match_cpu_via_pyramid() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 128,
        ..Default::default()
    };
    let frame = split_tone_frame(48, 32);
    let mut edits = Edits::default();
    edits.tone.shadows = 50.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("shadows mean abs delta = {delta:.3}");
    if delta > 8.0 {
        panic!("shadows GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: Some(80.0),
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("sharpen mean abs delta = {delta:.3}");
    if delta > 3.0 {
        panic!("sharpen GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_sharpen_masking_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: Some(80.0),
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 60.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("sharpen+masking mean abs delta = {delta:.3}");
    if delta > 3.0 {
        panic!("sharpen+masking GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_nr_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            luma_nr_amount: 50.0,
            luma_nr_detail: 40.0,
            luma_nr_contrast: 0.0,
            color_nr_amount: 50.0,
            color_nr_detail: 40.0,
            color_nr_smoothness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("nr mean abs delta = {delta:.3}");
    if delta > 4.0 {
        panic!("nr GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_nr_matches_cpu_with_preview_downsample() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 320,
        quality: false,
        ..Default::default()
    };
    let frame = fine_texture_frame(1024, 768);
    let edits = Edits {
        detail: DetailEdits {
            luma_nr_amount: 100.0,
            luma_nr_detail: 30.0,
            luma_nr_contrast: 0.0,
            color_nr_amount: 60.0,
            color_nr_detail: 40.0,
            color_nr_smoothness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("nr preview-downsample mean abs delta = {delta:.3}");
    if delta > 1.5 {
        panic!("nr preview GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_capture_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 256,
        quality: true,
        ..Default::default()
    };
    let mut frame = detail_frame(256, 192);
    frame.is_raw = true;
    frame.capture_sigma = Some(0.7);
    let flat = ColorEdits {
        dcp: DcpEdits {
            mode: DcpMode::Flat,
            ..Default::default()
        },
        ..Default::default()
    };
    let edits = Edits {
        color: flat.clone(),
        ..Default::default()
    };
    let off = Edits {
        color: flat,
        detail: DetailEdits {
            capture_sharpen: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_off = renderer.render(&frame, &off, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let (off_rgb, _, _) = decode_jpeg_rgb(&gpu_off.bytes);
    let effect = mean_abs_delta(&gpu_rgb, &off_rgb);
    eprintln!("capture sharpen effect = {effect:.3}");
    if effect < 0.5 {
        panic!("capture sharpen had no visible effect: {effect:.3}");
    }
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("capture sharpen mean abs delta = {delta:.3}");
    if delta > 1.0 {
        panic!("capture sharpen GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_vignette_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        effects: EffectsEdits {
            vignette_amount: -50.0,
            vignette_midpoint: 40.0,
            vignette_feather: 60.0,
            vignette_roundness: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("vignette mean abs delta = {delta:.3}");
    if delta > 2.0 {
        panic!("vignette GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_grain_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        effects: EffectsEdits {
            grain_amount: 50.0,
            grain_size: 25.0,
            grain_roughness: 50.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("grain mean abs delta = {delta:.3}");
    if delta > 6.0 {
        panic!("grain GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_effects_with_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: Some(60.0),
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        effects: EffectsEdits {
            vignette_amount: -40.0,
            grain_amount: 30.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    assert_eq!(gpu.width, cpu.width);
    assert_eq!(gpu.height, cpu.height);
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("sharpen+effects mean abs delta = {delta:.3}");
    if delta > 6.0 {
        panic!("sharpen+effects GPU/CPU mean abs delta too high: {delta:.3}");
    }
}

#[test]
fn gpu_linear_histogram_changes_with_vignette() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let frame = synthetic_frame(96, 64);
    let neutral = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let edited = renderer
        .render(
            &frame,
            &Edits {
                effects: EffectsEdits {
                    vignette_amount: -80.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            &opts,
        )
        .unwrap();
    let nh = neutral.linear_histogram.expect("neutral linear histogram");
    let eh = edited.linear_histogram.expect("edited linear histogram");
    let diff: i64 =
        nh.l.iter()
            .zip(eh.l.iter())
            .map(|(a, b)| (*a as i64 - *b as i64).abs())
            .sum();
    if diff == 0 {
        panic!("linear histogram unchanged after vignette");
    }
}
