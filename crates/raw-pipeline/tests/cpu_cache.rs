use raw_pipeline::{
    CpuRenderer, cpu,
    edits::{CropRect, Edits},
    frame::{OutputFormat, RenderOptions},
};

mod common;

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

    vec![
        ("identity", Edits::default(), preview()),
        ("exposure", exposure, preview()),
        ("tone", tone, preview()),
        ("wb", wb, preview()),
        ("detail", detail, preview()),
        ("no_capture_sharpen", no_capture, preview()),
        ("geometry", geometry, preview()),
        ("clarity_dehaze", clarity, preview()),
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
    let Some(frame) = common::first_fixture_frame() else {
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
fn cpu_cache_reuse_across_tone_edits() {
    let Some(frame) = common::first_fixture_frame() else {
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
