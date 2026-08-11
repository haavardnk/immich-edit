mod common;

use common::{decode_jpeg_rgb, detail_frame, mean_abs_delta, synthetic_frame, try_renderer};
use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    Vec2f,
};
use raw_pipeline::frame::RenderOptions;

#[test]
fn gpu_masks_match_cpu_within_tolerance() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };

    for invert in [false, true] {
        let layer = MaskLayer {
            id: "L1".into(),
            name: "".into(),
            enabled: true,
            color: "#ff3b30".into(),
            amount: 1.0,
            invert,
            components: vec![
                MaskComponent {
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
                },
                MaskComponent {
                    id: "c2".into(),
                    enabled: true,
                    mode: MaskComponentMode::Subtract,
                    invert: false,
                    kind: MaskComponentKind::Radial {
                        center: Vec2f { x: 0.25, y: 0.5 },
                        radius_xy: Vec2f { x: 0.2, y: 0.2 },
                        feather: 0.3,
                    },
                    source: MaskSource::Manual,
                    generated: None,
                },
            ],
            edits: MaskedEdits {
                exposure_ev: Some(1.2),
                brightness: Some(25.0),
                saturation: Some(30.0),
                contrast: Some(20.0),
                wb_temp: Some(15.0),
                ..Default::default()
            },
        };

        let edits = Edits {
            masks: vec![layer],
            ..Default::default()
        };

        let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
        if cpu_out.width != gpu_out.width || cpu_out.height != gpu_out.height {
            panic!(
                "dim mismatch CPU {}x{} vs GPU {}x{}",
                cpu_out.width, cpu_out.height, gpu_out.width, gpu_out.height
            );
        }
        let (cpu_rgb, cw, ch) = decode_jpeg_rgb(&cpu_out.bytes);
        let (gpu_rgb, gw, gh) = decode_jpeg_rgb(&gpu_out.bytes);
        if (cw, ch) != (gw, gh) {
            panic!("decoded dim mismatch {cw}x{ch} vs {gw}x{gh}");
        }
        let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
        eprintln!("masks parity (invert={invert}): mean abs delta = {delta:.3}");
        if delta > 1.5 {
            panic!("CPU vs GPU mask drift with invert={invert}: {delta:.3} > 1.5");
        }
    }
}

#[test]
fn gpu_brush_masks_match_cpu_within_tolerance() {
    use raw_pipeline::mask_raster::{MaskRaster, RasterMap};
    use std::sync::Arc;

    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let w: u32 = 32;
    let h: u32 = 32;
    let mut bytes = vec![0u8; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            if x >= w / 2 {
                bytes[(y * w + x) as usize] = 255;
            }
        }
    }
    let raster = Arc::new(MaskRaster::new(w, h, bytes).unwrap());
    let mut rasters = RasterMap::new();
    rasters.insert("brush_a".into(), raster);

    let opts = RenderOptions {
        max_edge: 96,
        rasters,
        ..Default::default()
    };

    let layer = MaskLayer {
        id: "L1".into(),
        name: "".into(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "b1".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::Brush {
                raster_id: "brush_a".into(),
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            exposure_ev: Some(1.5),
            saturation: Some(25.0),
            ..Default::default()
        },
    };

    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };

    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    if cpu_out.width != gpu_out.width || cpu_out.height != gpu_out.height {
        panic!(
            "dim mismatch CPU {}x{} vs GPU {}x{}",
            cpu_out.width, cpu_out.height, gpu_out.width, gpu_out.height
        );
    }
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu_out.bytes);
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu_out.bytes);
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("brush masks parity: mean abs delta = {delta:.3}");
    if delta > 2.0 {
        panic!("CPU vs GPU brush mask drift: {delta:.3} > 2.0");
    }
}

#[test]
fn gpu_masked_presence_matches_cpu_and_changes_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let layer = MaskLayer {
        id: "L1".into(),
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
                feather: 0.2,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            texture: Some(80.0),
            clarity: Some(60.0),
            ..Default::default()
        },
    };
    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };
    let plain = Edits::default();

    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
    let (cpu_rgb, _, _) = decode_jpeg_rgb(&cpu_out.bytes);
    let (cpu_plain_rgb, _, _) = decode_jpeg_rgb(&cpu_plain.bytes);
    let cpu_effect = mean_abs_delta(&cpu_rgb, &cpu_plain_rgb);
    eprintln!("masked presence cpu effect = {cpu_effect:.3}");
    if cpu_effect < 0.5 {
        panic!("masked texture and clarity had no effect on the CPU path: {cpu_effect:.3}");
    }

    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu_out.bytes);
    let (gpu_plain_rgb, _, _) = decode_jpeg_rgb(&gpu_plain.bytes);
    let gpu_effect = mean_abs_delta(&gpu_rgb, &gpu_plain_rgb);
    eprintln!("masked presence gpu effect = {gpu_effect:.3}");
    if gpu_effect < 0.5 {
        panic!("masked texture and clarity had no effect on the GPU path: {gpu_effect:.3}");
    }

    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("masked presence parity = {delta:.3}");
    if delta > 8.0 {
        panic!("CPU vs GPU masked presence drift: {delta:.3} > 8.0");
    }
}

#[test]
fn gpu_masked_sharpen_matches_cpu_and_changes_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = detail_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let layer = MaskLayer {
        id: "L1".into(),
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
                feather: 0.2,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            sharpen: Some(120.0),
            ..Default::default()
        },
    };
    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };
    let plain = Edits::default();

    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
    let (cpu_rgb, w, h) = decode_jpeg_rgb(&cpu_out.bytes);
    let (cpu_plain_rgb, _, _) = decode_jpeg_rgb(&cpu_plain.bytes);
    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
    let (gpu_rgb, _, _) = decode_jpeg_rgb(&gpu_out.bytes);
    let (gpu_plain_rgb, _, _) = decode_jpeg_rgb(&gpu_plain.bytes);

    let region_delta = |a: &[u8], b: &[u8], from: usize, to: usize| -> f64 {
        let samples: Vec<usize> = (0..h)
            .flat_map(|y| (from..to).flat_map(move |x| (0..3).map(move |c| (y * w + x) * 3 + c)))
            .collect();
        let sum: f64 = samples
            .iter()
            .map(|&i| (a[i] as f64 - b[i] as f64).abs())
            .sum();
        sum / samples.len() as f64
    };

    let cpu_masked = region_delta(&cpu_rgb, &cpu_plain_rgb, w * 3 / 4, w);
    let cpu_clear = region_delta(&cpu_rgb, &cpu_plain_rgb, 0, w / 8);
    eprintln!("masked sharpen cpu masked = {cpu_masked:.3} clear = {cpu_clear:.3}");
    if cpu_masked < 0.5 {
        panic!("masked sharpen had no effect on the CPU path: {cpu_masked:.3}");
    }
    if cpu_clear > 0.2 {
        panic!("masked sharpen leaked outside the mask on the CPU path: {cpu_clear:.3}");
    }

    let gpu_masked = region_delta(&gpu_rgb, &gpu_plain_rgb, w * 3 / 4, w);
    let gpu_clear = region_delta(&gpu_rgb, &gpu_plain_rgb, 0, w / 8);
    eprintln!("masked sharpen gpu masked = {gpu_masked:.3} clear = {gpu_clear:.3}");
    if gpu_masked < 0.5 {
        panic!("masked sharpen had no effect on the GPU path: {gpu_masked:.3}");
    }
    if gpu_clear > 0.5 {
        panic!("masked sharpen leaked outside the mask on the GPU path: {gpu_clear:.3}");
    }

    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("masked sharpen parity = {delta:.3}");
    if delta > 3.0 {
        panic!("CPU vs GPU masked sharpen drift: {delta:.3} > 3.0");
    }
}

#[test]
fn gpu_range_masks_match_cpu_within_tolerance() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let layer = MaskLayer {
        id: "L1".into(),
        name: String::new(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert: false,
        components: vec![
            MaskComponent {
                id: "color".into(),
                enabled: true,
                mode: MaskComponentMode::Add,
                invert: false,
                kind: MaskComponentKind::ColorRange {
                    sample_rgb: [0.65, 0.45, 0.35],
                    tolerance: 0.25,
                    softness: 0.15,
                },
                source: MaskSource::Manual,
                generated: None,
            },
            MaskComponent {
                id: "luma".into(),
                enabled: true,
                mode: MaskComponentMode::Intersect,
                invert: false,
                kind: MaskComponentKind::LumaRange {
                    min: 0.15,
                    max: 0.85,
                    softness: 0.15,
                },
                source: MaskSource::Manual,
                generated: None,
            },
        ],
        edits: MaskedEdits {
            exposure_ev: Some(0.8),
            saturation: Some(20.0),
            ..Default::default()
        },
    };
    let edits = Edits {
        masks: vec![layer],
        ..Default::default()
    };
    let opts = RenderOptions {
        max_edge: 96,
        ..Default::default()
    };
    let cpu_out = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu_out = renderer.render(&frame, &edits, &opts).unwrap();
    let (cpu_rgb, cw, ch) = decode_jpeg_rgb(&cpu_out.bytes);
    let (gpu_rgb, gw, gh) = decode_jpeg_rgb(&gpu_out.bytes);
    if (cw, ch) != (gw, gh) {
        panic!("range mask decoded dim mismatch {cw}x{ch} vs {gw}x{gh}");
    }
    let delta = mean_abs_delta(&cpu_rgb, &gpu_rgb);
    eprintln!("range masks: mean abs delta = {delta:.3}");
    if delta > 2.5 {
        panic!("CPU vs GPU range mask drift: {delta:.3} > 2.5");
    }
}
