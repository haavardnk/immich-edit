mod common;

use common::{
    ParityLedger, detail_frame, fine_texture_frame, haze_frame, mean_abs_delta, require_same_dims,
    rgb8_opts, split_tone_frame, stripe_frame, synthetic_frame, try_renderer,
};
use raw_pipeline::edits::{ColorEdits, DcpEdits, DcpMode, DetailEdits, Edits, EffectsEdits};
use raw_pipeline::frame::{OutputFormat, RenderOptions};

#[test]
fn gpu_presence_sliders_match_cpu_via_fallback() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = stripe_frame(48, 32);
    let mut edits = Edits::default();
    edits.basic.texture = 30.0;
    edits.basic.clarity = 20.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    require_same_dims("presence", &cpu, &gpu);
    let mut ledger = ParityLedger::new("presence");
    ledger.check("texture+clarity", &cpu.bytes, &gpu.bytes, 0.1);
    ledger.finish();
}

#[test]
fn gpu_dehaze_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = haze_frame(96, 64);
    let mut edits = Edits::default();
    edits.basic.dehaze = 60.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    require_same_dims("dehaze", &cpu, &gpu);
    let mut ledger = ParityLedger::new("dehaze");
    ledger.check("dehaze+60", &cpu.bytes, &gpu.bytes, 0.15);
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
    ledger.check("dehaze+presence", &cpu.bytes, &gpu.bytes, 0.25);
    ledger.finish();
}

#[test]
fn gpu_shadows_match_cpu_via_pyramid() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(128);
    let frame = split_tone_frame(48, 32);
    let mut edits = Edits::default();
    edits.tone.shadows = 50.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    require_same_dims("shadows", &cpu, &gpu);
    let mut ledger = ParityLedger::new("shadows");
    ledger.check("shadows+50", &cpu.bytes, &gpu.bytes, 0.1);
    ledger.finish();
}

#[test]
fn gpu_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);
    let frame = synthetic_frame(96, 64);
    let mut ledger = ParityLedger::new("sharpen");
    for (label, masking, limit) in [("plain", 0.0, 0.2), ("masking+60", 60.0, 0.1)] {
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

#[test]
fn gpu_nr_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);
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
    require_same_dims("nr", &cpu, &gpu);
    let mut ledger = ParityLedger::new("nr");
    ledger.check("native", &cpu.bytes, &gpu.bytes, 0.3);
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
    require_same_dims("nr-preview", &cpu, &gpu);
    let mut ledger = ParityLedger::new("nr");
    ledger.check("preview-downsample", &cpu.bytes, &gpu.bytes, 0.25);
    ledger.finish();
}

#[test]
fn gpu_capture_sharpen_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = RenderOptions {
        max_edge: 256,
        quality: true,
        output: OutputFormat::Rgb8,
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
    require_same_dims("capture-sharpen", &cpu, &gpu);
    let effect = mean_abs_delta(&gpu.bytes, &gpu_off.bytes);
    eprintln!("capture sharpen effect = {effect:.3}");
    if effect < 0.5 {
        panic!("capture sharpen had no visible effect: {effect:.3}");
    }
    let mut ledger = ParityLedger::new("capture-sharpen");
    ledger.check("sigma0.7", &cpu.bytes, &gpu.bytes, 0.2);
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
            0.1,
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
            "grain",
            0.1,
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
            0.15,
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
    ledger.finish();
}

#[test]
fn gpu_linear_histogram_changes_with_vignette() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);
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
