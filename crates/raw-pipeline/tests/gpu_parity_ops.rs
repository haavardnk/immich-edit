mod common;

use common::{
    ParityLedger, any_fixture, require_same_dims, rgb_frame, rgb8_opts, synthetic_frame,
    try_renderer,
};
use raw_pipeline::edits::{
    BasicEdits, CropRect, CurvePoint, CurvePoints, CurvesEdits, GeometryEdits, LensEdits, ToneEdits,
};
use raw_pipeline::frame::RenderOptions;
use raw_pipeline::{decode, edits::Edits};

#[test]
fn gpu_identity_render_jpeg() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        eprintln!("no fixture, skipping");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = RenderOptions {
        max_edge: 512,
        ..Default::default()
    };
    let out = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    if out.bytes.len() < 1000 {
        panic!("jpeg too small ({} bytes)", out.bytes.len());
    }
    if &out.bytes[..2] != b"\xff\xd8" {
        panic!("not jpeg SOI marker");
    }
    if out.width.max(out.height) > 512 {
        panic!("max edge exceeded {}x{}", out.width, out.height);
    }
    if out.renderer != "gpu" {
        panic!("renderer label: {}", out.renderer);
    }
}

#[test]
fn gpu_exposure_brightens() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = rgb8_opts(256);

    let base = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let bright = Edits {
        basic: BasicEdits {
            exposure_ev: 2.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let bumped = renderer.render(&frame, &bright, &opts).unwrap();

    let mean_base: f64 = base
        .histogram
        .l
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / base.histogram.l.iter().sum::<u32>().max(1) as f64;
    let mean_bumped: f64 = bumped
        .histogram
        .l
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / bumped.histogram.l.iter().sum::<u32>().max(1) as f64;

    if mean_bumped <= mean_base {
        panic!("exposure did not brighten: {mean_base} -> {mean_bumped}");
    }
}

#[test]
fn gpu_rotate_swaps_dims() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = rgb8_opts(512);

    let a = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let rotated = Edits {
        geometry: GeometryEdits {
            rotate: 90,
            ..Default::default()
        },
        ..Default::default()
    };
    let b = renderer.render(&frame, &rotated, &opts).unwrap();

    let landscape_a = a.width >= a.height;
    let landscape_b = b.width >= b.height;
    if landscape_a == landscape_b {
        panic!(
            "rotate did not swap orientation: {}x{} -> {}x{}",
            a.width, a.height, b.width, b.height
        );
    }
}

#[test]
fn gpu_reports_sensor_source_dims_at_small_max_edge() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = rgb8_opts(256);

    let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();

    if (gpu.source_w, gpu.source_h) != (cpu.source_w, cpu.source_h) {
        panic!(
            "source dims disagree: gpu {}x{} cpu {}x{}",
            gpu.source_w, gpu.source_h, cpu.source_w, cpu.source_h
        );
    }
    if gpu.source_w.max(gpu.source_h) <= gpu.width.max(gpu.height) {
        panic!(
            "source dims collapsed to the working texture: {}x{} for a {}x{} render",
            gpu.source_w, gpu.source_h, gpu.width, gpu.height
        );
    }
}

#[test]
fn gpu_matches_cpu_within_tolerance() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);

    let cases: &[(&str, f64, Edits)] = &[
        ("identity", 1.0, Edits::default()),
        (
            "exposure+1.5",
            1.1,
            Edits {
                basic: BasicEdits {
                    exposure_ev: 1.5,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "saturation+50",
            1.0,
            Edits {
                basic: BasicEdits {
                    saturation: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "contrast+30",
            2.0,
            Edits {
                basic: BasicEdits {
                    contrast: 30.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "brightness+35",
            2.0,
            Edits {
                basic: BasicEdits {
                    brightness: 35.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "whites+50",
            1.5,
            Edits {
                tone: ToneEdits {
                    whites: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "blacks+50",
            1.5,
            Edits {
                tone: ToneEdits {
                    blacks: 50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "highlights-100",
            2.5,
            Edits {
                tone: ToneEdits {
                    highlights: -100.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "rotate10+crop",
            2.5,
            Edits {
                geometry: GeometryEdits {
                    rotate_angle: 10.0,
                    crop: Some(CropRect {
                        x: 0.15,
                        y: 0.15,
                        w: 0.7,
                        h: 0.7,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "perspective+rotate+crop",
            2.5,
            Edits {
                geometry: GeometryEdits {
                    rotate_angle: 6.0,
                    crop: Some(CropRect {
                        x: 0.12,
                        y: 0.12,
                        w: 0.7,
                        h: 0.7,
                    }),
                    perspective: Some(raw_pipeline::perspective::PerspectiveEdits {
                        vertical: 45.0,
                        horizontal: -20.0,
                        aspect: 15.0,
                        corners: None,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "per_channel_curves",
            2.0,
            Edits {
                basic: BasicEdits {
                    curves: CurvesEdits {
                        composite: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.04 },
                                CurvePoint { x: 0.5, y: 0.55 },
                                CurvePoint { x: 1.0, y: 0.97 },
                            ],
                        },
                        r: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.0 },
                                CurvePoint { x: 0.5, y: 0.62 },
                                CurvePoint { x: 1.0, y: 1.0 },
                            ],
                        },
                        g: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.0 },
                                CurvePoint { x: 0.5, y: 0.42 },
                                CurvePoint { x: 1.0, y: 1.0 },
                            ],
                        },
                        b: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.0 },
                                CurvePoint { x: 0.5, y: 0.58 },
                                CurvePoint { x: 1.0, y: 1.0 },
                            ],
                        },
                        luma: CurvePoints {
                            points: vec![
                                CurvePoint { x: 0.0, y: 0.02 },
                                CurvePoint { x: 0.4, y: 0.45 },
                                CurvePoint { x: 1.0, y: 0.98 },
                            ],
                        },
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_distortion_barrel",
            4.0,
            Edits {
                lens: LensEdits {
                    profile_enabled: Some(true),
                    distortion_amount: 100.0,
                    k1: -0.1,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_vignette_brighten",
            3.0,
            Edits {
                lens: LensEdits {
                    profile_enabled: Some(true),
                    vignette_amount: 100.0,
                    vk1: -0.4,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_ca_red",
            4.0,
            Edits {
                lens: LensEdits {
                    ca_enabled: true,
                    ca_red_scale_x10000: 80.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "lens_combo",
            5.0,
            Edits {
                lens: LensEdits {
                    profile_enabled: Some(true),
                    ca_enabled: true,
                    distortion_amount: 100.0,
                    k1: -0.08,
                    vignette_amount: 100.0,
                    vk1: -0.3,
                    ca_red_scale_x10000: 50.0,
                    ca_blue_scale_x10000: -50.0,
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
    ];

    let mut ledger = ParityLedger::new("ops");
    for (label, limit, edits) in cases {
        let cpu = raw_pipeline::cpu::render(&frame, edits, &opts).unwrap();
        let gpu = renderer.render(&frame, edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, *limit);
    }
    ledger.finish();
}

#[test]
fn gpu_exif_orientation_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(256);
    let w: usize = 40;
    let h: usize = 30;
    let data = vec![0.5f32; w * h * 3];

    let orientations: &[((bool, bool, bool), &str)] = &[
        ((false, false, false), "Normal"),
        ((false, true, false), "HorizontalFlip"),
        ((false, false, true), "VerticalFlip"),
        ((false, true, true), "Rotate180"),
        ((true, false, false), "Transpose"),
        ((true, false, true), "Rotate90"),
        ((true, true, false), "Rotate270"),
        ((true, true, true), "Transverse"),
    ];

    let mut ledger = ParityLedger::new("orientation");
    for &(orient, label) in orientations {
        let mut frame = rgb_frame(w, h, data.clone());
        frame.orientation = orient;

        let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, 1.0);
    }
    ledger.finish();
}
