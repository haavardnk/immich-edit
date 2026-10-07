use raw_pipeline::GpuRenderer;
use raw_pipeline::decode;
use raw_pipeline::edits::{
    BasicEdits, Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource,
    MaskedEdits, Vec2f,
};
use raw_pipeline::frame::{
    BitDepth, OutputFormat, PngCompression, PreviewMode, RawFrame, RenderOptions, RenderedImage,
};
use raw_pipeline::tone::srgb_oetf_scalar;
use raw_pipeline_testkit::frames::{detail_frame, rgb_frame, step_edge_frame, synthetic_frame};
use raw_pipeline_testkit::gpu::try_renderer;
use raw_pipeline_testkit::parity::{ParityLedger, mean_abs_delta, require_same_dims};
use raw_pipeline_testkit::render::rgb8_opts;

const PRESENCE_DEHAZE: f64 = 0.3;

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

fn display_rgb8(image: &RenderedImage, opts: &RenderOptions) -> Vec<u8> {
    if matches!(opts.output, OutputFormat::Rgb8) {
        return image.bytes.clone();
    }
    decode::decode(&image.bytes)
        .unwrap()
        .data
        .iter()
        .map(|&v| (srgb_oetf_scalar(v).clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect()
}

struct PlanCase<'a> {
    label: &'a str,
    frame: &'a RawFrame,
    opts: &'a RenderOptions,
    components: Vec<MaskComponent>,
    edits: MaskedEdits,
    invert: bool,
    fast_tolerance: f64,
    presence_tolerance: f64,
    min_effect: f64,
}

fn check_both_plans(renderer: &GpuRenderer, case: PlanCase) {
    let mut ledger = ParityLedger::new("masks");
    for (plan, dehaze, tolerance) in [
        ("fast", 0.0, case.fast_tolerance),
        ("presence", PRESENCE_DEHAZE, case.presence_tolerance),
    ] {
        let basic = BasicEdits {
            dehaze,
            ..Default::default()
        };
        let masked = Edits {
            basic: basic.clone(),
            masks: vec![layer(
                case.components.clone(),
                case.edits.clone(),
                case.invert,
            )],
            ..Default::default()
        };
        let bare = Edits {
            basic,
            ..Default::default()
        };
        let label = format!("{}-{plan}", case.label);

        let cpu = raw_pipeline::cpu::render(case.frame, &masked, case.opts).unwrap();
        let cpu_bare = raw_pipeline::cpu::render(case.frame, &bare, case.opts).unwrap();
        let gpu = renderer.render(case.frame, &masked, case.opts).unwrap();
        let gpu_bare = renderer.render(case.frame, &bare, case.opts).unwrap();
        require_same_dims(&label, &cpu, &gpu);
        let cpu = display_rgb8(&cpu, case.opts);
        let cpu_bare = display_rgb8(&cpu_bare, case.opts);
        let gpu = display_rgb8(&gpu, case.opts);
        let gpu_bare = display_rgb8(&gpu_bare, case.opts);

        let cpu_effect = mean_abs_delta(&cpu, &cpu_bare);
        let gpu_effect = mean_abs_delta(&gpu, &gpu_bare);
        eprintln!("{label} cpu effect = {cpu_effect:.3} gpu effect = {gpu_effect:.3}");
        if cpu_effect < case.min_effect {
            panic!("{label}: the mask had no effect on the CPU path: {cpu_effect:.3}");
        }
        if gpu_effect < case.min_effect {
            panic!("{label}: the mask had no effect on the GPU path: {gpu_effect:.3}");
        }
        ledger.check(&label, &cpu, &gpu, tolerance);
    }
    ledger.finish();
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
                angle: 0.0,
            },
            source: MaskSource::Manual,
            generated: None,
        };
        check_both_plans(
            &renderer,
            PlanCase {
                label: if invert { "invert" } else { "normal" },
                frame: &frame,
                opts: &opts,
                components: vec![linear_component(0.4), radial],
                edits: MaskedEdits {
                    exposure_ev: Some(1.2),
                    brightness: Some(25.0),
                    saturation: Some(30.0),
                    contrast: Some(20.0),
                    wb_temp: Some(15.0),
                    ..Default::default()
                },
                invert,
                fast_tolerance: 0.07,
                presence_tolerance: 0.35,
                min_effect: 0.5,
            },
        );
    }
}

#[test]
fn gpu_mask_weight_image_matches_cpu_as_gray_weight() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        preview_mode: PreviewMode::MaskWeight {
            layer_id: "L1".into(),
        },
        ..rgb8_opts(96)
    };
    let mut edits = Edits {
        masks: vec![layer(
            vec![linear_component(0.4)],
            MaskedEdits {
                exposure_ev: Some(2.0),
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };
    edits.geometry.rotate = 90;
    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    require_same_dims("weight image", &cpu, &gpu);
    for (label, bytes) in [("cpu", &cpu.bytes), ("gpu", &gpu.bytes)] {
        if bytes
            .chunks_exact(3)
            .any(|px| px[0].abs_diff(px[1]) > 2 || px[0].abs_diff(px[2]) > 2)
        {
            panic!("{label}: the weight image is not gray");
        }
        let lo = bytes.iter().min().copied().unwrap_or(0);
        let hi = bytes.iter().max().copied().unwrap_or(0);
        if lo > 8 || hi < 247 {
            panic!("{label}: the weight image does not span the weight range: {lo}..{hi}");
        }
    }
    let mut ledger = ParityLedger::new("masks");
    ledger.check("weight image", &cpu.bytes, &gpu.bytes, 0.5);
    ledger.finish();
}

#[test]
fn gpu_masks_render_on_16bit_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = RenderOptions {
        max_edge: 96,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        ..Default::default()
    };
    check_both_plans(
        &renderer,
        PlanCase {
            label: "sixteen",
            frame: &frame,
            opts: &opts,
            components: vec![linear_component(0.4)],
            edits: MaskedEdits {
                exposure_ev: Some(1.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.35,
            presence_tolerance: 0.35,
            min_effect: 0.5,
        },
    );
}

#[test]
fn gpu_rotated_radial_matches_cpu_on_a_wide_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);
    let radial = MaskComponent {
        id: "r1".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::Radial {
            center: Vec2f { x: 0.5, y: 0.5 },
            radius_xy: Vec2f { x: 0.35, y: 0.12 },
            feather: 0.3,
            angle: 35.0,
        },
        source: MaskSource::Manual,
        generated: None,
    };
    check_both_plans(
        &renderer,
        PlanCase {
            label: "rotated",
            frame: &frame,
            opts: &opts,
            components: vec![radial],
            edits: MaskedEdits {
                exposure_ev: Some(1.2),
                saturation: Some(30.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.07,
            presence_tolerance: 0.35,
            min_effect: 0.5,
        },
    );
}

#[test]
fn gpu_diagonal_and_polygon_masks_match_cpu_on_a_wide_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);
    let diagonal = MaskComponent {
        kind: MaskComponentKind::Linear {
            p0: Vec2f { x: 0.2, y: 0.2 },
            p1: Vec2f { x: 0.8, y: 0.8 },
            feather: 0.6,
        },
        ..linear_component(0.0)
    };
    let polygon = MaskComponent {
        id: "p1".into(),
        mode: MaskComponentMode::Subtract,
        kind: MaskComponentKind::Polygon {
            points: vec![
                Vec2f { x: 0.55, y: 0.2 },
                Vec2f { x: 0.9, y: 0.3 },
                Vec2f { x: 0.8, y: 0.85 },
                Vec2f { x: 0.6, y: 0.7 },
            ],
            feather: 0.15,
        },
        ..linear_component(0.0)
    };
    check_both_plans(
        &renderer,
        PlanCase {
            label: "diagonal-polygon",
            frame: &frame,
            opts: &opts,
            components: vec![diagonal, polygon],
            edits: MaskedEdits {
                exposure_ev: Some(1.2),
                saturation: Some(30.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.07,
            presence_tolerance: 0.35,
            min_effect: 0.5,
        },
    );
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
    check_both_plans(
        &renderer,
        PlanCase {
            label: "brush",
            frame: &frame,
            opts: &opts,
            components: vec![brush],
            edits: MaskedEdits {
                exposure_ev: Some(1.5),
                saturation: Some(25.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.05,
            presence_tolerance: 0.35,
            min_effect: 0.5,
        },
    );
}

#[test]
fn gpu_masked_presence_matches_cpu_and_changes_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = detail_frame(96, 64);
    let opts = rgb8_opts(96);
    let presence_layer = Edits {
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
    let mut exposure_layer = Edits {
        masks: vec![layer(
            vec![linear_component(0.2)],
            MaskedEdits {
                exposure_ev: Some(1.5),
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };
    exposure_layer.basic.clarity = 60.0;
    let plain = Edits::default();
    let mut ledger = ParityLedger::new("masks");
    for (label, edits) in [
        ("masked-presence", presence_layer),
        ("masked-exposure-clarity", exposure_layer),
    ] {
        let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
        let cpu_plain = raw_pipeline::cpu::render(&frame, &plain, &opts).unwrap();
        let cpu_effect = mean_abs_delta(&cpu.bytes, &cpu_plain.bytes);
        eprintln!("{label} cpu effect = {cpu_effect:.3}");
        if cpu_effect < 0.5 {
            panic!("{label} had no effect on the CPU path: {cpu_effect:.3}");
        }

        let gpu = renderer.render(&frame, &edits, &opts).unwrap();
        let gpu_plain = renderer.render(&frame, &plain, &opts).unwrap();
        let gpu_effect = mean_abs_delta(&gpu.bytes, &gpu_plain.bytes);
        eprintln!("{label} gpu effect = {gpu_effect:.3}");
        if gpu_effect < 0.5 {
            panic!("{label} had no effect on the GPU path: {gpu_effect:.3}");
        }
        ledger.check(label, &cpu.bytes, &gpu.bytes, 0.07);
    }
    ledger.finish();
}

#[test]
fn gpu_masked_shadows_match_cpu_without_global_shadows() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let data = (0..64)
        .flat_map(|_| 0..96)
        .flat_map(|x| {
            let v = 0.01 + 0.12 * x as f32 / 95.0;
            [v, v * 0.9, v * 0.8]
        })
        .collect();
    let frame = rgb_frame(96, 64, data);
    let opts = rgb8_opts(96);
    check_both_plans(
        &renderer,
        PlanCase {
            label: "shadows",
            frame: &frame,
            opts: &opts,
            components: vec![linear_component(0.4)],
            edits: MaskedEdits {
                shadows: Some(80.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.07,
            presence_tolerance: 0.35,
            min_effect: 0.5,
        },
    );
}

#[test]
fn gpu_masked_sharpen_matches_cpu_and_changes_output() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = step_edge_frame(96, 64);
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
    ledger.check("masked-sharpen", &cpu.bytes, &gpu.bytes, 0.1);
    ledger.finish();
}

#[test]
fn gpu_masked_wb_matches_cpu_on_presence_plan() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = synthetic_frame(96, 64);
    let opts = rgb8_opts(96);
    // dehaze is what puts the render on the Presence plan, where the layer pass runs a
    // shader with no WhiteBalance stage.
    let build = |wb_temp: Option<f64>| Edits {
        basic: BasicEdits {
            dehaze: 0.3,
            ..Default::default()
        },
        masks: vec![layer(
            vec![linear_component(0.4)],
            MaskedEdits {
                exposure_ev: Some(0.5),
                wb_temp,
                ..Default::default()
            },
            false,
        )],
        ..Default::default()
    };
    let edits = build(Some(80.0));
    let no_wb = build(None);

    let cpu = raw_pipeline::cpu::render(&frame, &edits, &opts).unwrap();
    let cpu_no_wb = raw_pipeline::cpu::render(&frame, &no_wb, &opts).unwrap();
    let cpu_effect = mean_abs_delta(&cpu.bytes, &cpu_no_wb.bytes);
    eprintln!("masked presence wb cpu effect = {cpu_effect:.3}");
    if cpu_effect < 0.5 {
        panic!("masked white balance had no effect on the CPU path: {cpu_effect:.3}");
    }

    let gpu = renderer.render(&frame, &edits, &opts).unwrap();
    let gpu_no_wb = renderer.render(&frame, &no_wb, &opts).unwrap();
    let gpu_effect = mean_abs_delta(&gpu.bytes, &gpu_no_wb.bytes);
    eprintln!("masked presence wb gpu effect = {gpu_effect:.3}");
    if gpu_effect < 0.5 {
        panic!("masked white balance had no effect on the GPU path: {gpu_effect:.3}");
    }

    require_same_dims("masked-presence-wb", &cpu, &gpu);
    let mut ledger = ParityLedger::new("masks");
    ledger.check("masked-presence-wb", &cpu.bytes, &gpu.bytes, 0.35);
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
    check_both_plans(
        &renderer,
        PlanCase {
            label: "color+luma-range",
            frame: &frame,
            opts: &opts,
            components: vec![color, luma],
            edits: MaskedEdits {
                exposure_ev: Some(0.8),
                saturation: Some(20.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.07,
            presence_tolerance: 0.35,
            min_effect: 0.5,
        },
    );
}

#[test]
fn gpu_smoothed_luma_range_matches_cpu_on_a_textured_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let frame = step_edge_frame(2048, 96);
    let opts = rgb8_opts(2048);
    let luma = MaskComponent {
        id: "luma".into(),
        enabled: true,
        mode: MaskComponentMode::Add,
        invert: false,
        kind: MaskComponentKind::LumaRange {
            min: 0.45,
            max: 1.0,
            softness: 0.02,
        },
        source: MaskSource::Manual,
        generated: None,
    };
    check_both_plans(
        &renderer,
        PlanCase {
            label: "smoothed-luma-range",
            frame: &frame,
            opts: &opts,
            components: vec![luma],
            edits: MaskedEdits {
                exposure_ev: Some(1.0),
                ..Default::default()
            },
            invert: false,
            fast_tolerance: 0.05,
            presence_tolerance: 0.2,
            min_effect: 0.5,
        },
    );
}
