use std::sync::Arc;

use raw_pipeline::ai_denoise::DenoisePlane;
use raw_pipeline::edits::{
    BasicEdits, ColorEdits, DcpEdits, DcpMode, DetailEdits, Edits, EffectsEdits,
};
use raw_pipeline::frame::{OutputFormat, PreviewMode, RawFrame, RenderOptions};
use raw_pipeline_testkit::denoise::denoise_plane;
use raw_pipeline_testkit::frames::{
    as_raw, detail_frame, haze_frame, noisy_frame, split_tone_frame, step_edge_frame, stripe_frame,
    synthetic_bayer_frame, synthetic_frame,
};
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::parity::{ParityLedger, mean_abs_delta, require_same_dims};
use raw_pipeline_testkit::render::rgb8_opts;

const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

#[test]
fn gpu_presence_sliders_match_cpu_via_fallback() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = stripe_frame(48, 32);
    let mut ledger = ParityLedger::new("presence");
    for (label, exposure_ev) in [("texture+clarity", 0.0), ("texture+clarity-ev+1.5", 1.5)] {
        let mut edits = Edits::default();
        edits.basic.texture = 30.0;
        edits.basic.clarity = 20.0;
        edits.basic.exposure_ev = exposure_ev;
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.04);
    }
    ledger.finish();
}

#[test]
fn gpu_clarity_edge_guard_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(512);
    let frame = step_edge_frame(512, 384);
    let mut ledger = ParityLedger::new("presence");
    for (clarity, limit) in [(100.0, 0.06), (-60.0, 0.09)] {
        let mut edits = Edits::default();
        edits.basic.clarity = clarity;
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let label = format!("clarity{clarity:+}-edge");
        require_same_dims(&label, &cpu, &gpu);
        ledger.check(&label, &cpu.bytes, &gpu.bytes, limit);
    }
    ledger.finish();
}

#[test]
fn gpu_dehaze_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = haze_frame(96, 64);
    let mut ledger = ParityLedger::new("dehaze");
    for amount in [60.0, -60.0] {
        let mut edits = Edits::default();
        edits.basic.dehaze = amount;
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        require_same_dims("dehaze", &cpu, &gpu);
        ledger.check(&format!("dehaze{amount:+}"), &cpu.bytes, &gpu.bytes, 0.08);
    }
    ledger.finish();
}

#[test]
fn gpu_dehaze_matches_cpu_at_guided_filter_downscale() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(768);
    let frame = haze_frame(768, 640);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    require_same_dims("dehaze-downscale", &cpu, &gpu);
    if cpu.width.min(cpu.height) < 512 {
        panic!(
            "dehaze downscale case rendered at {}x{}, below the 512 guided-filter threshold",
            cpu.width, cpu.height
        );
    }
    let mut ledger = ParityLedger::new("dehaze");
    ledger.check("dehaze+60-large", &cpu.bytes, &gpu.bytes, 0.1);
    ledger.finish();
}

#[test]
fn gpu_dehaze_with_presence_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = haze_frame(96, 64);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;
    edits.basic.clarity = 80.0;
    edits.basic.texture = 80.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    require_same_dims("dehaze+presence", &cpu, &gpu);
    let mut ledger = ParityLedger::new("dehaze");
    ledger.check("dehaze+presence", &cpu.bytes, &gpu.bytes, 0.15);
    ledger.finish();
}

#[test]
fn gpu_shadows_match_cpu_via_pyramid() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = split_tone_frame(48, 32);
    let mut ledger = ParityLedger::new("shadows");
    for (label, exposure_ev, brightness, dehaze, clarity) in [
        ("shadows+50", 0.0, 0.0, 0.0, 0.0),
        ("shadows+50-ev+2", 2.0, 0.0, 0.0, 0.0),
        ("shadows+50-bright+60", 0.0, 60.0, 0.0, 0.0),
        ("shadows+50-dehaze+60", 0.0, 0.0, 60.0, 0.0),
        ("shadows+50-clarity+80", 0.0, 0.0, 0.0, 80.0),
    ] {
        let mut edits = Edits::default();
        edits.tone.shadows = 50.0;
        edits.basic.exposure_ev = exposure_ev;
        edits.basic.brightness = brightness;
        edits.basic.dehaze = dehaze;
        edits.basic.clarity = clarity;
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.04);
    }
    ledger.finish();
}

#[test]
fn gpu_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);
    let mut ledger = ParityLedger::new("sharpen");
    let cases = [
        ("plain", 96, 0.0, 0.15),
        ("masking+60", 96, 60.0, 0.05),
        ("half scale", 192, 0.0, 0.15),
    ];
    for (label, width, masking, limit) in cases {
        let frame = synthetic_frame(width, width * 2 / 3);
        let edits = Edits {
            detail: DetailEdits {
                sharpen_amount: Some(80.0),
                sharpen_radius: 1.0,
                sharpen_detail: 25.0,
                sharpen_masking: masking,
                ..Default::default()
            },
            ..Default::default()
        };
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, limit);
    }
    ledger.finish();
}

fn worst_overshoot(base: &[u8], sharp: &[u8], w: usize, h: usize) -> i32 {
    let mut worst = 0i32;
    for (y, x) in (0..h).flat_map(|y| (0..w).map(move |x| (y, x))) {
        let ys = [y.saturating_sub(1), y, (y + 1).min(h - 1)];
        let xs = [x.saturating_sub(1), x, (x + 1).min(w - 1)];
        for c in 0..3 {
            let mut lo = u8::MAX;
            let mut hi = u8::MIN;
            for (ny, nx) in ys.into_iter().flat_map(|ny| xs.map(move |nx| (ny, nx))) {
                let v = base[(ny * w + nx) * 3 + c];
                lo = lo.min(v);
                hi = hi.max(v);
            }
            let v = sharp[(y * w + x) * 3 + c] as i32;
            worst = worst.max(v - hi as i32).max(lo as i32 - v);
        }
    }
    worst
}

#[test]
fn sharpen_stays_inside_local_extrema_on_a_step_edge() {
    let opts = rgb8_opts(96);
    let frame = step_edge_frame(96, 64);
    let flat = Edits::default();
    let sharp = Edits {
        detail: DetailEdits {
            sharpen_amount: Some(150.0),
            sharpen_radius: 1.0,
            sharpen_detail: 100.0,
            sharpen_masking: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let cpu_flat = raw_pipeline::cpu::render(&frame, &flat, &opts).unwrap();
    let cpu_sharp = raw_pipeline::cpu::render(&frame, &sharp, &opts).unwrap();
    let w = cpu_flat.width as usize;
    let h = cpu_flat.height as usize;
    let mut renders = vec![("cpu", cpu_flat.bytes.clone(), cpu_sharp.bytes)];
    if let Some(renderer) = try_renderer() {
        let gpu_flat = renderer.render(&frame, &flat, &opts).unwrap();
        let gpu_sharp = renderer.render(&frame, &sharp, &opts).unwrap();
        require_same_dims("sharpen-overshoot", &cpu_flat, &gpu_sharp);
        renders.push(("gpu", gpu_flat.bytes, gpu_sharp.bytes));
    }
    for (label, flat_bytes, sharp_bytes) in renders {
        let effect = mean_abs_delta(&flat_bytes, &sharp_bytes);
        if effect < 0.5 {
            panic!("{label} sharpen had no visible effect: {effect:.3}");
        }
        let worst = worst_overshoot(&flat_bytes, &sharp_bytes, w, h);
        if worst > 2 {
            panic!("{label} sharpen overshot the local extrema by {worst}");
        }
    }
}

#[test]
fn gpu_sharpen_previews_match_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = detail_frame(128, 96);
    let edits = Edits {
        detail: DetailEdits {
            sharpen_amount: Some(80.0),
            sharpen_radius: 1.0,
            sharpen_detail: 25.0,
            sharpen_masking: 40.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut ledger = ParityLedger::new("sharpen-preview");
    for (label, mode, limit) in [
        ("mask", PreviewMode::SharpenMask, 1.0),
        ("radius", PreviewMode::SharpenRadius, 0.15),
        ("detail", PreviewMode::SharpenDetail, 0.4),
    ] {
        let opts = RenderOptions {
            preview_mode: mode,
            ..rgb8_opts(128)
        };
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, limit);
    }
    ledger.finish();
}

fn require_nr_effect(label: &str, plain: &[u8], denoised: &[u8]) {
    let effect = mean_abs_delta(plain, denoised);
    if effect < 1.0 {
        panic!("{label}: noise reduction barely changed the image: {effect:.3}");
    }
}

#[test]
fn gpu_nr_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(192);
    let frame = noisy_frame(192, 128);
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
    let plain = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    require_same_dims("nr", &cpu, &gpu);
    require_nr_effect("nr", &plain.bytes, &cpu.bytes);
    let mut ledger = ParityLedger::new("nr");
    ledger.check("native", &cpu.bytes, &gpu.bytes, 0.09);
    ledger.finish();
}

#[test]
fn gpu_nr_matches_cpu_with_preview_downsample() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 320,
        quality: false,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let frame = noisy_frame(1024, 768);
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
    let plain = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    require_same_dims("nr-preview", &cpu, &gpu);
    require_nr_effect("nr-preview", &plain.bytes, &cpu.bytes);
    let mut ledger = ParityLedger::new("nr");
    ledger.check("preview-downsample", &cpu.bytes, &gpu.bytes, 0.15);
    ledger.finish();
}

#[test]
fn gpu_capture_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
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
    let mut ledger = ParityLedger::new("capture-sharpen");
    let cases: [(&str, RawFrame, f32, f64); 3] = [
        ("sigma0.7", detail_frame(256, 192), 0.7, 0.0),
        ("sigma2-ragged", step_edge_frame(250, 190), 2.0, 0.0),
        ("corner-boost", detail_frame(256, 192), 0.7, 100.0),
    ];
    for (label, mut frame, sigma, boost) in cases {
        let opts = RenderOptions {
            max_edge: frame.meta.width.max(frame.meta.height) as u32,
            quality: true,
            output: OutputFormat::Rgb8,
            ..Default::default()
        };
        frame.meta.is_raw = true;
        frame.meta.capture_sigma = Some(sigma);
        let mut boosted = edits.clone();
        boosted.detail.capture_corner_boost = boost;
        let cpu = raw_pipeline::cpu::render(&frame, &boosted, &opts).unwrap();
        let gpu = renderer.render(&frame, &boosted, &opts).unwrap();
        let gpu_off = renderer.render(&frame, &off, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        let effect = mean_abs_delta(&gpu.bytes, &gpu_off.bytes);
        eprintln!("{label}: capture sharpen effect = {effect:.3}");
        if effect < 0.3 {
            panic!("{label}: capture sharpen had no visible effect: {effect:.3}");
        }
        if boost > 0.0 {
            let gpu_plain = renderer.render(&frame, &edits, &opts).unwrap();
            let boost_effect = mean_abs_delta(&gpu.bytes, &gpu_plain.bytes);
            eprintln!("{label}: corner boost effect = {boost_effect:.3}");
            if boost_effect < 0.07 {
                panic!("{label}: corner boost had no visible effect: {boost_effect:.3}");
            }
        }
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.09);
    }
    ledger.finish();
}

#[test]
fn gpu_effects_match_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);
    let frame = synthetic_frame(96, 64);

    let cases: &[(&str, f64, Edits)] = &[
        (
            "vignette",
            0.04,
            Edits {
                effects: EffectsEdits {
                    vignette_amount: -50.0,
                    vignette_midpoint: 40.0,
                    vignette_feather: 60.0,
                    vignette_roundness: 0.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "vignette-100",
            0.04,
            Edits {
                basic: BasicEdits {
                    exposure_ev: 1.5,
                    ..Default::default()
                },
                effects: EffectsEdits {
                    vignette_amount: -100.0,
                    vignette_midpoint: 10.0,
                    vignette_feather: 20.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "grain",
            0.04,
            Edits {
                effects: EffectsEdits {
                    grain_amount: 50.0,
                    grain_size: 25.0,
                    grain_roughness: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "sharpen+vignette+grain",
            0.1,
            Edits {
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
            },
        ),
    ];

    let mut ledger = ParityLedger::new("effects");
    for (label, limit, edits) in cases {
        let cpu = raw_pipeline::cpu::render(&frame, edits, &opts).unwrap();
        let gpu = renderer.render(&frame, edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, *limit);
    }
    let wide = synthetic_frame(384, 256);
    for (label, size) in [
        ("grain quarter scale fine", 0.0),
        ("grain quarter scale coarse", 100.0),
    ] {
        let edits = Edits {
            effects: EffectsEdits {
                grain_amount: 60.0,
                grain_size: size,
                grain_roughness: 50.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let cpu = raw_pipeline::cpu::render(&wide, &edits, &opts).unwrap();
        let gpu = renderer.render(&wide, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.08);
    }
    ledger.finish();
}

#[test]
fn gpu_linear_histogram_changes_with_vignette() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        histogram: true,
        ..rgb8_opts(96)
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

fn smoothed_plane(frame: &RawFrame, key: u64, lift: f32) -> Arc<DenoisePlane> {
    let w = frame.meta.width;
    let h = frame.meta.height;
    let rgb = match frame.cpp {
        3 => frame.data.clone(),
        _ => frame.data.iter().flat_map(|v| [*v; 3]).collect(),
    };
    let at = |x: isize, y: isize, c: usize| {
        let x = x.clamp(0, w as isize - 1) as usize;
        let y = y.clamp(0, h as isize - 1) as usize;
        rgb[(y * w + x) * 3 + c]
    };
    let smoothed: Vec<f32> = (0..w * h * 3)
        .map(|i| {
            let c = i % 3;
            let x = ((i / 3) % w) as isize;
            let y = ((i / 3) / w) as isize;
            let sum: f32 = (-2..=2)
                .flat_map(|dy| (-2..=2).map(move |dx| (dx, dy)))
                .map(|(dx, dy)| at(x + dx, y + dy, c))
                .sum();
            sum / 25.0 + lift
        })
        .collect();
    denoise_plane(frame, key, &smoothed)
}

fn ai_denoise_edits(strength: f64, presence: bool) -> Edits {
    let mut edits = Edits::default();
    edits.detail.ai_denoise = true;
    edits.detail.ai_denoise_strength = strength;
    edits.detail.sharpen_amount = Some(0.0);
    if presence {
        edits.detail.luma_nr_amount = 20.0;
    }
    edits
}

#[test]
fn gpu_ai_denoise_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let rgb = as_raw(noisy_frame(192, 128));
    let bayer = as_raw(synthetic_bayer_frame(1024, 768, "RGGB"));
    let odd = as_raw(synthetic_bayer_frame(1029, 769, "RGGB"));
    let xtrans = as_raw(synthetic_bayer_frame(1031, 769, XTRANS));
    let cases = [
        ("fast", &rgb, rgb8_opts(192), false, 0.7),
        ("presence", &rgb, rgb8_opts(192), true, 0.7),
        ("superpixel", &bayer, rgb8_opts(384), false, 0.7),
        ("superpixel-presence", &bayer, rgb8_opts(384), true, 0.7),
        ("superpixel-odd", &odd, rgb8_opts(384), false, 0.7),
        ("superpixel-odd-presence", &odd, rgb8_opts(384), true, 0.7),
        ("xtrans", &xtrans, rgb8_opts(256), false, 0.7),
        ("xtrans-presence", &xtrans, rgb8_opts(256), true, 0.7),
    ];
    let mut ledger = ParityLedger::new("ai_denoise");
    for (key, (label, frame, base, presence, limit)) in (11..).zip(cases) {
        let opts = RenderOptions {
            denoise: Some(smoothed_plane(frame, key, 0.0)),
            ..base
        };
        let edits = ai_denoise_edits(80.0, presence);
        let mut plain_edits = edits.clone();
        plain_edits.detail.ai_denoise = false;
        let plain = raw_pipeline::cpu::render(frame, &plain_edits, &opts).unwrap();
        let plain_gpu = renderer.render(frame, &plain_edits, &opts).unwrap();
        ledger.check(
            &format!("{label}-plain"),
            &plain.bytes,
            &plain_gpu.bytes,
            limit,
        );
        let cpu = raw_pipeline::cpu::render(frame, &edits, &opts).unwrap();
        let gpu = renderer.render(frame, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        let effect = mean_abs_delta(&plain.bytes, &cpu.bytes);
        if effect < 1.0 {
            panic!("{label}: AI denoise barely changed the cpu image: {effect:.3}");
        }
        let gpu_effect = mean_abs_delta(&plain_gpu.bytes, &gpu.bytes);
        if gpu_effect < 1.0 {
            panic!("{label}: AI denoise barely changed the gpu image: {gpu_effect:.3}");
        }
        ledger.check(label, &cpu.bytes, &gpu.bytes, limit);
        let zero_edits = ai_denoise_edits(0.0, presence);
        let zero = raw_pipeline::cpu::render(frame, &zero_edits, &opts).unwrap();
        if zero.bytes != plain.bytes {
            panic!("{label}: zero strength changed the cpu image");
        }
        let zero_gpu = renderer.render(frame, &zero_edits, &opts).unwrap();
        if zero_gpu.bytes != plain_gpu.bytes {
            panic!("{label}: zero strength changed the gpu image");
        }
    }
    ledger.finish();
}

#[test]
fn gpu_ai_denoise_detail_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let rgb = as_raw(noisy_frame(192, 128));
    let bayer = as_raw(synthetic_bayer_frame(1024, 768, "RGGB"));
    let cases = [
        ("fast", &rgb, rgb8_opts(192), false),
        ("presence", &rgb, rgb8_opts(192), true),
        ("superpixel", &bayer, rgb8_opts(384), false),
        ("superpixel-presence", &bayer, rgb8_opts(384), true),
    ];
    let mut ledger = ParityLedger::new("ai_denoise_detail");
    for (key, (label, frame, base, presence)) in (41..).zip(cases) {
        let opts = RenderOptions {
            denoise: Some(smoothed_plane(frame, key, 0.0)),
            ..base
        };
        let smooth = ai_denoise_edits(100.0, presence);
        let mut grainy = smooth.clone();
        grainy.detail.ai_denoise_detail = 70.0;
        let cpu_smooth = raw_pipeline::cpu::render(frame, &smooth, &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(frame, &grainy, &opts).unwrap();
        let gpu = renderer.render(frame, &grainy, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        let effect = mean_abs_delta(&cpu_smooth.bytes, &cpu.bytes);
        if effect < 0.5 {
            panic!("{label}: detail barely changed the cpu image: {effect:.3}");
        }
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.7);
    }
    ledger.finish();
}

#[test]
fn a_plane_that_does_not_match_the_frame_fails_on_both_renderers() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = as_raw(noisy_frame(96, 64));
    let other = as_raw(noisy_frame(64, 96));
    let opts = RenderOptions {
        denoise: Some(smoothed_plane(&other, 31, 0.0)),
        ..rgb8_opts(96)
    };
    for presence in [false, true] {
        let edits = ai_denoise_edits(80.0, presence);
        if raw_pipeline::cpu::render(&frame, &edits, &opts).is_ok() {
            panic!("presence {presence}: the cpu rendered with a mismatched plane");
        }
        if renderer.render(&frame, &edits, &opts).is_ok() {
            panic!("presence {presence}: the gpu rendered with a mismatched plane");
        }
    }
}

#[test]
fn gpu_ai_denoise_follows_a_new_plane() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = as_raw(noisy_frame(160, 96));
    let mut ledger = ParityLedger::new("ai_denoise");
    for presence in [false, true] {
        let edits = ai_denoise_edits(100.0, presence);
        let mut previous: Option<Vec<u8>> = None;
        for (key, lift) in [(21, 0.0), (22, 0.05)] {
            let opts = RenderOptions {
                denoise: Some(smoothed_plane(&frame, key, lift)),
                ..rgb8_opts(160)
            };
            let gpu = renderer.render(&frame, &edits, &opts).unwrap();
            let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
            ledger.check(
                &format!("plane-{key}-presence-{presence}"),
                &cpu.bytes,
                &gpu.bytes,
                0.7,
            );
            if previous.as_ref() == Some(&gpu.bytes) {
                panic!("presence {presence}: the gpu kept serving the previous plane");
            }
            previous = Some(gpu.bytes);
        }
    }
    ledger.finish();
}
