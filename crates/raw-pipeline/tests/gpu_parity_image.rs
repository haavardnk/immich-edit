mod common;

use common::{
    ParityLedger, any_fixture, mean_abs_laplacian, require_same_dims, rgb8_opts,
    synthetic_bayer_frame, synthetic_frame, try_renderer, warn_pixels,
};
use raw_pipeline::decode;
use raw_pipeline::edits::{BasicEdits, CropRect, Edits, GeometryEdits};
use raw_pipeline::frame::{OutputFormat, RenderOptions};

#[test]
fn gpu_demosaic_matches_cpu_mhc() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);

    let patterns: &[(&str, usize, usize)] = &[
        ("RGGB", 96, 64),
        ("BGGR", 88, 60),
        ("GRBG", 80, 56),
        ("GBRG", 72, 52),
    ];

    let mut ledger = ParityLedger::new("demosaic");
    for (cfa, w, h) in patterns {
        let frame = synthetic_bayer_frame(*w, *h, cfa);
        let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
        let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        require_same_dims(cfa, &cpu, &gpu);
        ledger.check(cfa, &cpu.bytes, &gpu.bytes, 0.06);
    }
    ledger.finish();
}

#[test]
fn gpu_xtrans_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(96);

    const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";
    let sizes: &[(usize, usize)] = &[(96, 66), (86, 58)];

    let mut ledger = ParityLedger::new("xtrans");
    for (w, h) in sizes {
        let frame = synthetic_bayer_frame(*w, *h, XTRANS);
        let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
        let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
        let label = format!("{w}x{h}");
        require_same_dims(&label, &cpu, &gpu);
        ledger.check(&label, &cpu.bytes, &gpu.bytes, 0.1);
    }
    ledger.finish();
}

#[test]
fn gpu_xtrans_fixture_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf");
    let Ok(bytes) = std::fs::read(&path) else {
        eprintln!("no X-Trans fixture; skipping");
        return;
    };
    let frame = decode::decode(&bytes).unwrap();
    let opts = rgb8_opts(4096);
    let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    require_same_dims("fixture", &cpu, &gpu);
    let mut ledger = ParityLedger::new("xtrans");
    ledger.check("X-T2", &cpu.bytes, &gpu.bytes, 0.45);
    ledger.finish();
}

#[test]
fn gpu_matches_cpu_for_clipping_warnings() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    for (exposure, saturation, color) in [
        (3.0, 0.0, raw_pipeline::warn::HIGHLIGHT_WARN_RGB),
        (-14.0, 0.0, raw_pipeline::warn::SHADOW_WARN_RGB),
        (0.0, 100.0, raw_pipeline::warn::GAMUT_WARN_RGB),
    ] {
        let edits = Edits {
            basic: BasicEdits {
                exposure_ev: exposure,
                saturation,
                ..Default::default()
            },
            ..Default::default()
        };
        let opts = RenderOptions {
            max_edge: 96,
            output: OutputFormat::Rgb8,
            clip_warn: true,
            gamut_warn: true,
            ..Default::default()
        };
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let c = warn_pixels(&cpu.bytes, color);
        let g = warn_pixels(&gpu.bytes, color);
        if c.is_empty() {
            panic!("cpu produced no {color:?} warning pixels; test is vacuous");
        }
        let only_cpu = c.iter().filter(|i| !g.contains(i)).count();
        let only_gpu = g.iter().filter(|i| !c.contains(i)).count();
        let drift = (only_cpu + only_gpu) as f64 / c.len() as f64;
        eprintln!(
            "{color:?}: cpu={} gpu={} disagree={} ({drift:.4})",
            c.len(),
            g.len(),
            only_cpu + only_gpu
        );
        if drift > 0.01 {
            panic!("warning drift for {color:?}: {drift:.4} > 0.01");
        }
    }
}

#[test]
fn gpu_downscale_preserves_detail_like_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let Some(path) = any_fixture() else {
        eprintln!("no fixture, skipping");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    let opts = rgb8_opts(512);

    let cases: &[(&str, Edits)] = &[
        ("full", Edits::default()),
        (
            "crop50",
            Edits {
                geometry: GeometryEdits {
                    crop: Some(CropRect {
                        x: 0.25,
                        y: 0.25,
                        w: 0.5,
                        h: 0.5,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "crop50-origin",
            Edits {
                geometry: GeometryEdits {
                    crop: Some(CropRect {
                        x: 0.0,
                        y: 0.0,
                        w: 0.5,
                        h: 0.5,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "crop-full-width",
            Edits {
                geometry: GeometryEdits {
                    crop: Some(CropRect {
                        x: 0.0,
                        y: 0.0,
                        w: 1.0,
                        h: 0.5,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
        (
            "rotate90+crop",
            Edits {
                geometry: GeometryEdits {
                    rotate: 90,
                    crop: Some(CropRect {
                        x: 0.3,
                        y: 0.1,
                        w: 0.45,
                        h: 0.6,
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        ),
    ];

    let mut failed: Vec<String> = Vec::new();
    for (label, edits) in cases {
        let cpu = raw_pipeline::cpu::render(&frame, edits, &opts).unwrap();
        let gpu = renderer.render(&frame, edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        let w = cpu.width as usize;
        let h = cpu.height as usize;
        let cpu_detail = mean_abs_laplacian(&cpu.bytes, w, h);
        let gpu_detail = mean_abs_laplacian(&gpu.bytes, w, h);
        let ratio = gpu_detail / cpu_detail;
        eprintln!(
            "PARITY detail/{label} {w}x{h} cpu={cpu_detail:.3} gpu={gpu_detail:.3} ratio={ratio:.3}"
        );
        if !(0.85..=1.20).contains(&ratio) {
            failed.push(format!("{label}: {ratio:.3}"));
        }
    }
    if !failed.is_empty() {
        panic!(
            "gpu/cpu detail ratio outside 0.85..1.20: {}",
            failed.join("; ")
        );
    }
}
