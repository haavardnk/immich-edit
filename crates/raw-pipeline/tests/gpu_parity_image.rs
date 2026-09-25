mod common;

use common::{
    ParityLedger, any_fixture, mean_abs_laplacian, require_same_dims, rgb_frame, rgb8_opts,
    synthetic_bayer_frame, synthetic_frame, try_renderer, warn_pixels,
};
use raw_pipeline::decode;
use raw_pipeline::edits::{BasicEdits, CropRect, Edits, GeometryEdits};
use raw_pipeline::frame::{
    Align, BitDepth, OutputFormat, OutputSharpen, PngCompression, RenderOptions, SharpenLevel,
    SharpenMedia, Watermark, WatermarkAnchor, WatermarkImage,
};

fn ramp_red_u16(png: &[u8]) -> Vec<u16> {
    let frame = decode::decode(png).unwrap();
    frame
        .data
        .chunks_exact(3)
        .map(|px| (px[0].clamp(0.0, 1.0) * 65535.0).round() as u16)
        .collect()
}

#[test]
fn gpu_16bit_output_keeps_more_than_8_bit_levels() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let w = 512;
    let h = 8;
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / (w - 1) as f32;
            let i = (y * w + x) * 3;
            data[i] = u;
            data[i + 1] = u;
            data[i + 2] = u;
        }
    }
    let frame = rgb_frame(w, h, data);
    let opts = RenderOptions {
        max_edge: 1024,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };
    let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    require_same_dims("ramp16", &cpu, &gpu);

    let g = ramp_red_u16(&gpu.bytes);
    let c = ramp_red_u16(&cpu.bytes);
    let distinct: std::collections::BTreeSet<u16> = g.iter().copied().collect();
    if distinct.len() <= 256 {
        panic!(
            "gpu 16-bit ramp has only {} distinct levels; display path is 8-bit",
            distinct.len()
        );
    }
    let delta = g
        .iter()
        .zip(&c)
        .map(|(a, b)| (*a as i32 - *b as i32).unsigned_abs())
        .max()
        .unwrap();
    if delta > 600 {
        panic!("gpu vs cpu 16-bit ramp differs by {delta} levels");
    }
}

#[test]
fn gpu_16bit_lut_pass_reads_and_writes_16_bit() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let w = 512;
    let h = 8;
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let u = x as f32 / (w - 1) as f32;
            let i = (y * w + x) * 3;
            data[i] = u;
            data[i + 1] = u;
            data[i + 2] = u;
        }
    }
    let frame = rgb_frame(w, h, data);
    let lut = raw_pipeline::Lut3d::parse_cube(tint_lut_cube(16).as_bytes()).unwrap();
    let mut luts: raw_pipeline::LutMap = std::collections::HashMap::new();
    luts.insert("test".to_string(), std::sync::Arc::new(lut));
    let opts = RenderOptions {
        max_edge: 1024,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        luts,
        ..Default::default()
    };
    let mut edits = Edits::default();
    edits.color.lut_3d.lut_id = Some("test".to_string());
    edits.color.lut_3d.amount = 100.0;

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let plain = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    if gpu.bytes == plain.bytes {
        panic!("lut pass did not run; test is vacuous");
    }
    let g = ramp_red_u16(&gpu.bytes);
    let distinct: std::collections::BTreeSet<u16> = g.iter().copied().collect();
    if distinct.len() <= 256 {
        panic!(
            "gpu 16-bit lut ramp has only {} distinct levels",
            distinct.len()
        );
    }
}

fn tint_lut_cube(size: usize) -> String {
    let last = (size - 1) as f32;
    let mut s = format!("LUT_3D_SIZE {size}\n");
    for b in 0..size {
        for g in 0..size {
            for r in 0..size {
                s.push_str(&format!(
                    "{} {} {}\n",
                    (r as f32 / last * 1.1).clamp(0.0, 1.0),
                    g as f32 / last,
                    (b as f32 / last * 0.85).clamp(0.0, 1.0)
                ));
            }
        }
    }
    s
}

#[test]
fn gpu_16bit_dcp_finish_writes_16_bit() {
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
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };
    let mut edits = Edits::default();
    edits.color.dcp.mode = raw_pipeline::edits::DcpMode::Flat;
    let with_dcp = renderer.render(&frame, &edits, &opts).unwrap();
    edits.color.dcp.mode = raw_pipeline::edits::DcpMode::Off;
    let without = renderer.render(&frame, &edits, &opts).unwrap();
    if with_dcp.bytes == without.bytes {
        panic!("dcp finish pass did not run; test is vacuous");
    }
    let g = ramp_red_u16(&with_dcp.bytes);
    let distinct: std::collections::BTreeSet<u16> = g.iter().copied().collect();
    if distinct.len() <= 256 {
        panic!(
            "gpu 16-bit dcp output has only {} distinct levels",
            distinct.len()
        );
    }
}

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
        ledger.check(cfa, &cpu.bytes, &gpu.bytes, 0.04);
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
        ledger.check(&label, &cpu.bytes, &gpu.bytes, 0.05);
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
fn gpu_enlarged_export_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let edits = Edits {
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
    };
    let opts = RenderOptions {
        max_edge: 240,
        enlarge: true,
        output: OutputFormat::Rgb8,
        quality: true,
        ..Default::default()
    };
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    require_same_dims("enlarge", &cpu, &gpu);
    if (cpu.width, cpu.height) != (240, 160) {
        panic!(
            "enlarged crop is {}x{}, want 240x160",
            cpu.width, cpu.height
        );
    }
    let native = raw_pipeline::cpu::render(
        &frame,
        &edits,
        &RenderOptions {
            enlarge: false,
            ..rgb8_opts(240)
        },
    )
    .unwrap();
    if (native.width, native.height) != (48, 32) {
        panic!("crop without enlarge is {}x{}", native.width, native.height);
    }
    let mut ledger = ParityLedger::new("enlarge");
    ledger.check("2.5x", &cpu.bytes, &gpu.bytes, 0.05);
    ledger.finish();
}

#[test]
fn gpu_output_sharpened_export_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let sharpen = OutputSharpen {
        media: SharpenMedia::Matte { ppi: 300 },
        level: SharpenLevel::High,
    };
    let opts = RenderOptions {
        output_sharpen: Some(sharpen),
        ..rgb8_opts(96)
    };
    let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    require_same_dims("output sharpen", &cpu, &gpu);
    let plain_cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &rgb8_opts(96)).unwrap();
    let plain_gpu = renderer
        .render(&frame, &Edits::default(), &rgb8_opts(96))
        .unwrap();
    if plain_cpu.bytes == cpu.bytes {
        panic!("output sharpening left the export unchanged");
    }
    let mut ledger = ParityLedger::new("output sharpen");
    let plain = ledger.check("plain", &plain_cpu.bytes, &plain_gpu.bytes, 0.05);
    // One shared CPU unsharp mask (k = 1.6) runs on both readbacks; it can widen their gap by at most 1 + 2k.
    ledger.check("matte high", &cpu.bytes, &gpu.bytes, plain * 4.2 + 0.005);
    ledger.finish();
}

#[test]
fn gpu_watermarked_export_matches_cpu() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let rgba = (0..16 * 16)
        .flat_map(|i| [255, 32, 32, ((i % 16) * 17) as u8])
        .collect();
    let watermark = Watermark {
        image: std::sync::Arc::new(WatermarkImage {
            width: 16,
            height: 16,
            rgba,
        }),
        size: 0.5,
        opacity: 0.8,
        anchor: WatermarkAnchor {
            x: Align::End,
            y: Align::End,
        },
        inset: 0.05,
    };
    let opts = RenderOptions {
        watermark: Some(watermark),
        ..rgb8_opts(96)
    };
    let cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &opts).unwrap();
    let gpu = renderer.render(&frame, &Edits::default(), &opts).unwrap();
    require_same_dims("watermark", &cpu, &gpu);
    let plain_cpu = raw_pipeline::cpu::render(&frame, &Edits::default(), &rgb8_opts(96)).unwrap();
    let plain_gpu = renderer
        .render(&frame, &Edits::default(), &rgb8_opts(96))
        .unwrap();
    if plain_cpu.bytes == cpu.bytes {
        panic!("the watermark left the export unchanged");
    }
    let mut ledger = ParityLedger::new("watermark");
    let plain = ledger.check("plain", &plain_cpu.bytes, &plain_gpu.bytes, 0.05);
    // The composite is a convex blend towards the same mark on both readbacks, so it only narrows their gap.
    ledger.check("bottom right", &cpu.bytes, &gpu.bytes, plain + 0.002);
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
        if !(0.95..=1.05).contains(&ratio) {
            failed.push(format!("{label}: {ratio:.3}"));
        }
    }
    if !failed.is_empty() {
        panic!(
            "gpu/cpu detail ratio outside 0.95..1.05: {}",
            failed.join("; ")
        );
    }
}
