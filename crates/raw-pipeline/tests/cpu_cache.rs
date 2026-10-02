use raw_pipeline::{
    CpuRenderer, cpu,
    edits::{
        CropRect, Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer,
        MaskSource, MaskedEdits, Vec2f,
    },
    frame::{OutputFormat, RenderOptions},
};
use raw_pipeline_testkit::fixtures::first_fixture_frame;
use raw_pipeline_testkit::frames::{rgb_frame, synthetic_frame};

fn variants() -> Vec<(&'static str, Edits, RenderOptions)> {
    let preview = || RenderOptions {
        max_edge: 900,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };

    let mut exposure = Edits::default();
    exposure.basic.exposure_ev = 0.8;

    let mut tone = Edits::default();
    tone.basic.contrast = 35.0;
    tone.tone.shadows = 40.0;
    tone.basic.saturation = -20.0;

    let mut wb = Edits::default();
    wb.basic.wb_temp = 30.0;
    wb.basic.wb_tint = -15.0;

    let mut detail = Edits::default();
    detail.detail.luma_nr_amount = 40.0;
    detail.detail.sharpen_amount = Some(60.0);

    let mut no_capture = Edits::default();
    no_capture.detail.capture_sharpen = false;

    let mut corner_boost = Edits::default();
    corner_boost.detail.capture_corner_boost = 80.0;

    let mut geometry = Edits::default();
    geometry.geometry.rotate = 90;
    geometry.geometry.crop = Some(CropRect {
        x: 0.1,
        y: 0.15,
        w: 0.6,
        h: 0.5,
    });

    let mut clarity = Edits::default();
    clarity.basic.clarity = 45.0;
    clarity.basic.dehaze = 30.0;

    let masked_wb = Edits {
        masks: vec![MaskLayer {
            id: "warm".into(),
            name: String::new(),
            enabled: true,
            color: "#ff3b30".into(),
            amount: 1.0,
            invert: false,
            components: vec![MaskComponent {
                id: "c1".into(),
                enabled: true,
                mode: MaskComponentMode::Add,
                invert: false,
                kind: MaskComponentKind::Linear {
                    p0: Vec2f { x: 0.0, y: 0.5 },
                    p1: Vec2f { x: 1.0, y: 0.5 },
                    feather: 0.4,
                },
                source: MaskSource::Manual,
                generated: None,
            }],
            edits: MaskedEdits {
                wb_temp: Some(40.0),
                ..Default::default()
            },
        }],
        ..Default::default()
    };

    vec![
        ("identity", Edits::default(), preview()),
        ("exposure", exposure, preview()),
        ("tone", tone, preview()),
        ("wb", wb, preview()),
        ("detail", detail, preview()),
        ("no_capture_sharpen", no_capture, preview()),
        ("capture_corner_boost", corner_boost, preview()),
        ("geometry", geometry, preview()),
        ("clarity_dehaze", clarity, preview()),
        ("masked_wb", masked_wb, preview()),
        (
            "quality",
            Edits::default(),
            RenderOptions {
                max_edge: 65535,
                quality: true,
                output: OutputFormat::Rgb8,
                ..Default::default()
            },
        ),
    ]
}

#[test]
fn cpu_cache_matches_uncached() {
    let Some(frame) = first_fixture_frame() else {
        eprintln!("no fixtures decoded; skipping");
        return;
    };
    let renderer = CpuRenderer::new();
    for (name, edits, options) in variants() {
        let expected = cpu::render(&frame, &edits, &options).unwrap();
        for pass in 0..2 {
            let got = renderer.render(&frame, &edits, &options).unwrap();
            assert_eq!(
                got.bytes, expected.bytes,
                "{name} pass {pass}: cached bytes differ"
            );
            assert_eq!(got.width, expected.width, "{name} pass {pass}: width");
            assert_eq!(got.height, expected.height, "{name} pass {pass}: height");
            assert_eq!(
                got.source_w, expected.source_w,
                "{name} pass {pass}: source_w"
            );
            assert_eq!(
                got.source_h, expected.source_h,
                "{name} pass {pass}: source_h"
            );
        }
    }
}

#[test]
fn cpu_cache_hits_on_every_display_tick() {
    let Some(frame) = first_fixture_frame() else {
        eprintln!("no fixtures decoded; skipping");
        return;
    };
    let options = RenderOptions {
        max_edge: 900,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    fn exposure(step: usize) -> Edits {
        let mut edits = Edits::default();
        edits.basic.exposure_ev = step as f64 * 0.05;
        edits
    }
    fn white_balance(step: usize) -> Edits {
        let mut edits = Edits::default();
        edits.detail.luma_nr_amount = 40.0;
        edits.basic.wb_temp = step as f64 * 3.0;
        edits
    }
    for (label, tick) in [
        ("exposure", exposure as fn(usize) -> Edits),
        ("white balance with noise reduction", white_balance),
    ] {
        let renderer = CpuRenderer::new();
        let misses: Vec<usize> = (0..20)
            .filter(|step| {
                let edits = tick(*step);
                let image = renderer.render(&frame, &edits, &options).unwrap();
                image.timings.iter().any(|t| t.stage == "demosaic")
            })
            .collect();
        if misses != [0] {
            panic!(
                "{label}: sensor cache should miss only on the first tick, missed on {misses:?}"
            );
        }
    }
}

#[test]
fn a_frame_in_a_reused_allocation_is_not_served_from_cache() {
    let options = RenderOptions {
        max_edge: 96,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let edits = Edits::default();
    let renderer = CpuRenderer::new();
    let first = synthetic_frame(96, 64);
    renderer.render(&first, &edits, &options).unwrap();
    let mut data = first.data;
    data.reverse();
    let second = rgb_frame(96, 64, data);

    let got = renderer.render(&second, &edits, &options).unwrap();
    let expected = cpu::render(&second, &edits, &options).unwrap();

    assert_eq!(
        got.bytes, expected.bytes,
        "a frame in a freed frame's allocation rendered from the old frame's cache"
    );
}

#[test]
fn cpu_cache_reuse_across_tone_edits() {
    let Some(frame) = first_fixture_frame() else {
        eprintln!("no fixtures decoded; skipping");
        return;
    };
    let options = RenderOptions {
        max_edge: 900,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let renderer = CpuRenderer::new();
    for step in 0..6 {
        let mut edits = Edits::default();
        edits.basic.exposure_ev = step as f64 * 0.2;
        let expected = cpu::render(&frame, &edits, &options).unwrap();
        let got = renderer.render(&frame, &edits, &options).unwrap();
        assert_eq!(
            got.bytes, expected.bytes,
            "step {step}: cached bytes differ"
        );
    }
}
