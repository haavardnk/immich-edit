mod common;

use common::{
    ParityLedger, detail_frame, mean_abs_delta, require_same_dims, rgb8_opts, synthetic_frame,
    try_renderer,
};
use raw_pipeline::edits::{
    Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource, MaskedEdits,
    Vec2f,
};
use raw_pipeline::frame::{OutputFormat, RenderOptions};

fn layer(components: Vec<MaskComponent>, edits: MaskedEdits, invert: bool) -> MaskLayer {
    MaskLayer {
        id: "L1".into(),
        name: String::new(),
        enabled: true,
        color: "#ff3b30".into(),
        amount: 1.0,
        invert,
        components,
        edits,
    }
}

fn linear_component(feather: f32) -> MaskComponent {
    MaskComponent {
        id: "c1".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Linear {
            p0: Vec2f { x: 0.0, y: 0.5 },
            p1: Vec2f { x: 1.0, y: 0.5 },
            feather,
        },
        source: MaskSource::Manual,
        generated: None,
    }
}

#[test]
fn gpu_masks_match_cpu_within_tolerance() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);

    let mut ledger = ParityLedger::new("masks");
    for invert in [false, true] {
        let radial = MaskComponent {
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
        };
        let edits = Edits {
            masks: vec![layer(
                vec![linear_component(0.4), radial],
                MaskedEdits {
                    exposure_ev: Some(1.2),
                    brightness: Some(25.0),
                    saturation: Some(30.0),
                    contrast: Some(20.0),
                    wb_temp: Some(15.0),
                    ..Default::default()
                },
                invert,
            )],
            ..Default::default()
        };

        let label = if invert { "invert" } else { "normal" };
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        require_same_dims(label, &cpu, &gpu);
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.3);
    }
    ledger.finish();
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
        output: OutputFormat::Rgb8,
        rasters,
        ..Default::default()
    };

    let brush = MaskComponent {
        id: "b1".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Brush {
            raster_id: "brush_a".into(),
        },
        source: MaskSource::Manual,
        generated: None,
    };
    let edits = Edits {
        masks: vec![layer(
            vec![brush],
            MaskedEdits {
                exposure_ev: Some(1.5),
                saturation: Some(25.0),
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };

    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    require_same_dims("brush", &cpu, &gpu);
    let mut ledger = ParityLedger::new("masks");
    ledger.check("brush", &cpu.bytes, &gpu.bytes, 0.1);
    ledger.finish();
}

#[test]
fn gpu_masked_presence_matches_cpu_and_changes_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = detail_frame(96, 64);
    let opts = rgb8_opts(96);
    let edits = Edits {
        masks: vec![layer(
            vec![linear_component(0.2)],
            MaskedEdits {
                texture: Some(80.0),
                clarity: Some(60.0),
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };
    let plain = Edits::default();

    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
    let cpu_effect = mean_abs_delta(&cpu.bytes, &cpu_plain.bytes);
    eprintln!("masked presence cpu effect = {cpu_effect:.3}");
    if cpu_effect < 0.5 {
        panic!("masked texture and clarity had no effect on the CPU path: {cpu_effect:.3}");
    }

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
    let gpu_effect = mean_abs_delta(&gpu.bytes, &gpu_plain.bytes);
    eprintln!("masked presence gpu effect = {gpu_effect:.3}");
    if gpu_effect < 0.5 {
        panic!("masked texture and clarity had no effect on the GPU path: {gpu_effect:.3}");
    }

    let mut ledger = ParityLedger::new("masks");
    ledger.check("masked-presence", &cpu.bytes, &gpu.bytes, 0.1);
    ledger.finish();
}

#[test]
fn gpu_masked_sharpen_matches_cpu_and_changes_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = detail_frame(96, 64);
    let opts = rgb8_opts(96);
    let edits = Edits {
        masks: vec![layer(
            vec![linear_component(0.2)],
            MaskedEdits {
                sharpen: Some(120.0),
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };
    let plain = Edits::default();

    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
    require_same_dims("masked-sharpen", &cpu, &gpu);

    let w = cpu.width as usize;
    let h = cpu.height as usize;
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

    let cpu_masked = region_delta(&cpu.bytes, &cpu_plain.bytes, w * 3 / 4, w);
    let cpu_clear = region_delta(&cpu.bytes, &cpu_plain.bytes, 0, w / 8);
    eprintln!("masked sharpen cpu masked = {cpu_masked:.3} clear = {cpu_clear:.3}");
    if cpu_masked < 0.5 {
        panic!("masked sharpen had no effect on the CPU path: {cpu_masked:.3}");
    }
    if cpu_clear > 0.2 {
        panic!("masked sharpen leaked outside the mask on the CPU path: {cpu_clear:.3}");
    }

    let gpu_masked = region_delta(&gpu.bytes, &gpu_plain.bytes, w * 3 / 4, w);
    let gpu_clear = region_delta(&gpu.bytes, &gpu_plain.bytes, 0, w / 8);
    eprintln!("masked sharpen gpu masked = {gpu_masked:.3} clear = {gpu_clear:.3}");
    if gpu_masked < 0.5 {
        panic!("masked sharpen had no effect on the GPU path: {gpu_masked:.3}");
    }
    if gpu_clear > 0.5 {
        panic!("masked sharpen leaked outside the mask on the GPU path: {gpu_clear:.3}");
    }

    let mut ledger = ParityLedger::new("masks");
    ledger.check("masked-sharpen", &cpu.bytes, &gpu.bytes, 0.4);
    ledger.finish();
}

#[test]
fn gpu_range_masks_match_cpu_within_tolerance() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);
    let color = MaskComponent {
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
    };
    let luma = MaskComponent {
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
    };
    let edits = Edits {
        masks: vec![layer(
            vec![color, luma],
            MaskedEdits {
                exposure_ev: Some(0.8),
                saturation: Some(20.0),
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };

    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    require_same_dims("range", &cpu, &gpu);
    let mut ledger = ParityLedger::new("masks");
    ledger.check("color+luma-range", &cpu.bytes, &gpu.bytes, 0.3);
    ledger.finish();
}
