use raw_pipeline::edits::{
    BasicEdits, CropRect, CurvePoint, CurvePoints, CurvesEdits, GeometryEdits, LensEdits,
    RetouchMode, RetouchStroke, ToneEdits, Vec2f,
};
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::{decode, edits::Edits};
use raw_pipeline_testkit::fixtures::any_fixture;
use raw_pipeline_testkit::frames::{rgb_frame, synthetic_frame};
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::parity::{ParityLedger, mean_abs_delta, require_same_dims};
use raw_pipeline_testkit::render::rgb8_opts;

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
    let opts = RenderOptions {
        histogram: true,
        ..rgb8_opts(256)
    };

    let base = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let bright = Edits {
        basic: BasicEdits {
            exposure_ev: 2.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let bumped = renderer.render(&frame, &bright, &opts).unwrap();

    let base_hist = base.histogram.expect("base histogram requested");
    let bumped_hist = bumped.histogram.expect("bumped histogram requested");
    let mean_base: f64 = base_hist
        .l
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / base_hist.l.iter().sum::<u32>().max(1) as f64;
    let mean_bumped: f64 = bumped_hist
        .l
        .iter()
        .enumerate()
        .map(|(i, &n)| i as f64 * n as f64)
        .sum::<f64>()
        / bumped_hist.l.iter().sum::<u32>().max(1) as f64;

    if mean_bumped <= mean_base {
        panic!("exposure did not brighten: {mean_base} -> {mean_bumped}");
    }
}

fn blemish_frame(w: usize, h: usize, blemish_px: f32) -> RawFrame {
    let data = (0..w * h)
        .flat_map(|i| {
            let x = (i % w) as f32 + 0.5;
            let y = (i / w) as f32 + 0.5;
            let d2 = (x - 0.3 * w as f32).powi(2) + (y - 0.4 * h as f32).powi(2);
            let blemish = if blemish_px > 0.0 {
                0.2 * (-d2 / (2.0 * blemish_px * blemish_px)).exp()
            } else {
                0.0
            };
            let v = 0.3 + 0.2 * x / w as f32 + 0.1 * (y / h as f32).powi(2) - blemish;
            [v, v * 0.8, v * 0.6]
        })
        .collect();
    rgb_frame(w, h, data)
}

#[test]
fn gpu_heal_removes_the_blemish_like_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(160);
    let heal = Edits {
        retouch: vec![RetouchStroke {
            id: "h".into(),
            mode: RetouchMode::Heal,
            points: vec![Vec2f { x: 0.3, y: 0.4 }],
            radius: 0.15,
            hardness: 1.0,
            opacity: 1.0,
            source: Vec2f { x: 0.7, y: 0.6 },
            enabled: true,
        }],
        ..Default::default()
    };
    let frame = blemish_frame(160, 120, 6.0);
    let clean = renderer
        .render(&blemish_frame(160, 120, 0.0), &Edits::default(), &opts)
        .unwrap();
    let blemished = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &heal, &opts).unwrap();
    let gpu = renderer.render(&frame, &heal, &opts).unwrap();
    require_same_dims("heal", &cpu, &gpu);
    let before = mean_abs_delta(&blemished.bytes, &clean.bytes);
    for (label, out) in [("cpu", &cpu), ("gpu", &gpu)] {
        let residual = mean_abs_delta(&out.bytes, &clean.bytes) / before;
        eprintln!("heal {label} residual = {:.1}%", residual * 100.0);
        if residual > 0.15 {
            panic!("{label}: {:.0}% of the blemish remains", residual * 100.0);
        }
    }
    let mut ledger = ParityLedger::new("retouch");
    ledger.check("heal", &cpu.bytes, &gpu.bytes, 0.5);
    ledger.finish();
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
fn source_dims_stay_in_the_mask_frame_through_a_quarter_turn() {
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);
    let gpu = try_renderer();
    for rotate in [0, 90, 180, 270] {
        let edits = Edits {
            geometry: GeometryEdits {
                rotate,
                ..Default::default()
            },
            ..Default::default()
        };
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        if (cpu.source_w, cpu.source_h) != (96, 64) {
            panic!(
                "cpu rotate {rotate}: source {}x{}",
                cpu.source_w, cpu.source_h
            );
        }
        let Some(renderer) = &gpu else { continue };
        let out = renderer.render(&frame, &edits, &opts).unwrap();
        if (out.source_w, out.source_h) != (96, 64) {
            panic!(
                "gpu rotate {rotate}: source {}x{}",
                out.source_w, out.source_h
            );
        }
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
        ("identity", 0.04, Edits::default()),
        (
            "exposure+1.5",
            0.02,
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
            0.05,
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
            0.04,
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
            0.04,
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
            0.04,
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
            0.04,
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
            0.05,
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
            0.04,
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
            0.04,
            Edits {
                geometry: GeometryEdits {
                    rotate_angle: 6.0,
                    crop: Some(CropRect {
                        x: 0.12,
                        y: 0.12,
                        w: 0.7,
                        h: 0.7,
                    }),
                    perspective: Some(raw_pipeline::geom::perspective::PerspectiveEdits {
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
            0.04,
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
            0.07,
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
            0.07,
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
            0.05,
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
            0.09,
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
        frame.meta.orientation = orient;

        let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.01);
    }
    ledger.finish();
}
