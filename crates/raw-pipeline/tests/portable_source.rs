mod common;

use common::{first_fixture_frame, haze_frame, mean_abs_delta, rgb8_opts, try_renderer};
use raw_pipeline::CpuRenderer;
use raw_pipeline::GpuRenderer;
use raw_pipeline::edits::{
    CropRect, DetailEdits, Edits, GeometryEdits, MaskComponent, MaskComponentKind,
    MaskComponentMode, MaskLayer, MaskSource, MaskedEdits, Vec2f,
};
use raw_pipeline::frame::{RawFrame, RenderOptions};
use raw_pipeline::source::{self, LinearKind, SourceImage};

fn layer() -> MaskLayer {
    MaskLayer {
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
            wb_temp: Some(20.0),
            texture: Some(30.0),
            exposure_ev: Some(0.3),
            ..Default::default()
        },
    }
}

fn display_edits() -> Edits {
    let mut edits = Edits::default();
    edits.basic.exposure_ev = 0.4;
    edits.basic.contrast = 20.0;
    edits.basic.wb_temp = 15.0;
    edits.basic.wb_tint = -10.0;
    edits.basic.dehaze = 25.0;
    edits.basic.texture = 30.0;
    edits.basic.clarity = -20.0;
    edits.basic.vibrance = 15.0;
    edits.tone.shadows = 40.0;
    edits.tone.highlights = -30.0;
    edits.color.hsl.bands[2].sat = 25.0;
    edits.color.color_grade.shadows.sat = 20.0;
    edits.detail.sharpen_amount = Some(60.0);
    edits.effects.vignette_amount = -30.0;
    edits.effects.grain_amount = 20.0;
    edits.geometry.flip_h = true;
    edits.masks = vec![layer()];
    edits
}

fn all_edits() -> Edits {
    let display = display_edits();
    Edits {
        detail: DetailEdits {
            luma_nr_amount: 35.0,
            color_nr_amount: 25.0,
            ..display.detail
        },
        geometry: GeometryEdits {
            crop: Some(CropRect {
                x: 0.1,
                y: 0.05,
                w: 0.8,
                h: 0.85,
            }),
            ..display.geometry.clone()
        },
        ..display
    }
}

fn frames() -> Vec<(&'static str, RawFrame)> {
    let mut rotated = haze_frame(112, 160);
    rotated.meta.orientation = (true, false, true);
    let mut out = vec![("haze", haze_frame(160, 112)), ("rotated", rotated)];
    if let Some(frame) = first_fixture_frame() {
        out.push(("fixture", frame));
    }
    out
}

fn wire(image: &SourceImage) -> Vec<u8> {
    source::encode(image).unwrap()
}

fn gpu_source(
    renderer: &GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> SourceImage {
    renderer
        .render_source(frame, &edits.sensor_stage(), opts, None)
        .unwrap()
        .image
}

#[test]
fn a_source_survives_the_wire_and_renders_like_its_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(256);
    for (label, frame) in frames() {
        for (case, edits) in [("display", display_edits()), ("all", all_edits())] {
            let bytes = wire(&gpu_source(&renderer, &frame, &edits, &opts));
            let decoded = source::decode(&bytes).unwrap();
            if decoded.header.kind != LinearKind::PostWb || decoded.header.atmosphere.is_none() {
                panic!("{label} {case}: a source must be white balanced and carry an atmosphere");
            }
            let uploaded = renderer.upload_source(&decoded).unwrap();
            if wire(&renderer.read_source(&uploaded, None, None).unwrap()) != bytes {
                panic!("{label} {case}: uploading and reading back changed the source");
            }
            let from_frame = renderer.render(&frame, &edits, &opts).unwrap();
            let from_wire = renderer.render(&uploaded, &edits, &opts).unwrap();
            if from_frame.bytes != from_wire.bytes {
                panic!(
                    "{label} {case}: the wire source moved the render by {:.4}",
                    mean_abs_delta(&from_frame.bytes, &from_wire.bytes)
                );
            }
        }
    }
}

#[test]
fn a_source_stays_close_to_fast_plan_renders() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let opts = rgb8_opts(256);
    let mut exposure = Edits::default();
    exposure.basic.exposure_ev = 0.5;
    for (label, frame) in frames() {
        for edits in [Edits::default(), exposure.clone()] {
            let image = gpu_source(&renderer, &frame, &edits, &opts);
            let uploaded = renderer.upload_source(&image).unwrap();
            let from_frame = renderer.render(&frame, &edits, &opts).unwrap();
            let from_source = renderer.render(&uploaded, &edits, &opts).unwrap();
            let delta = mean_abs_delta(&from_frame.bytes, &from_source.bytes);
            if delta > 1.0 {
                panic!("{label}: the source drifted {delta:.3} from a fast plan render");
            }
        }
    }
}

#[test]
fn display_edits_do_not_reach_the_source() {
    let cpu = CpuRenderer::new();
    let gpu = try_renderer();
    let opts = rgb8_opts(256);
    for (label, frame) in frames() {
        let full = all_edits();
        let subset = full.sensor_stage();
        let cpu_full = wire(&cpu.render_source(&frame, &full, &opts, None).unwrap().image);
        let cpu_subset = wire(
            &cpu.render_source(&frame, &subset, &opts, None)
                .unwrap()
                .image,
        );
        if cpu_full != cpu_subset {
            panic!("{label}: a field outside Edits::sensor_stage changed the cpu source");
        }
        let Some(gpu) = gpu.as_ref() else {
            continue;
        };
        let gpu_full = wire(&gpu.render_source(&frame, &full, &opts, None).unwrap().image);
        let gpu_subset = wire(
            &gpu.render_source(&frame, &subset, &opts, None)
                .unwrap()
                .image,
        );
        if gpu_full != gpu_subset {
            panic!("{label}: a field outside Edits::sensor_stage changed the gpu source");
        }
    }
}

#[test]
fn a_cpu_source_renders_like_the_gpu_frame() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let cpu = CpuRenderer::new();
    let opts = rgb8_opts(256);
    for (label, frame) in frames() {
        for (case, edits) in [("display", display_edits()), ("all", all_edits())] {
            let image = cpu
                .render_source(&frame, &edits.sensor_stage(), &opts, None)
                .unwrap()
                .image;
            let uploaded = renderer
                .upload_source(&source::decode(&wire(&image)).unwrap())
                .unwrap();
            let from_frame = renderer.render(&frame, &edits, &opts).unwrap();
            let from_cpu = renderer.render(&uploaded, &edits, &opts).unwrap();
            if (from_frame.width, from_frame.height) != (from_cpu.width, from_cpu.height) {
                panic!("{label} {case}: the cpu source rendered at different dims");
            }
            let delta = mean_abs_delta(&from_frame.bytes, &from_cpu.bytes);
            if delta > 2.0 {
                panic!("{label} {case}: the cpu source drifted {delta:.3} from the gpu render");
            }
        }
    }
}

fn tile_opts(max_edge: u32, roi: CropRect) -> RenderOptions {
    RenderOptions {
        roi: Some(roi),
        ..rgb8_opts(max_edge)
    }
}

fn tile_cases() -> Vec<(&'static str, Edits)> {
    let mut tilted = all_edits();
    tilted.geometry.rotate_angle = 3.0;
    vec![("display", display_edits()), ("tilted", tilted)]
}

#[test]
fn a_windowed_source_renders_the_tile_of_the_full_source() {
    let Some(renderer) = try_renderer() else {
        return;
    };
    let cpu = CpuRenderer::new();
    let mut turned = haze_frame(800, 1200);
    turned.meta.orientation = (true, false, true);
    let roi = CropRect {
        x: 0.4,
        y: 0.35,
        w: 0.2,
        h: 0.25,
    };
    for (label, frame) in [("haze", haze_frame(1200, 800)), ("turned", turned)] {
        let long = frame.meta.width.max(frame.meta.height) as u32;
        for (case, edits) in tile_cases() {
            let sensor = edits.sensor_stage();
            let tile = tile_opts(long / 4, roi);
            let sources = [
                (
                    "gpu",
                    renderer.render_source(&frame, &sensor, &tile, None),
                    renderer.render_source(&frame, &sensor, &rgb8_opts(long), None),
                ),
                (
                    "cpu",
                    cpu.render_source(&frame, &sensor, &tile, None),
                    cpu.render_source(&frame, &sensor, &rgb8_opts(long), None),
                ),
            ];
            for (kind, windowed, full) in sources {
                let windowed = source::decode(&wire(&windowed.unwrap().image)).unwrap();
                let full = source::decode(&wire(&full.unwrap().image)).unwrap();
                let Some(window) = windowed.header.window else {
                    panic!("{label} {case} {kind}: a small tile was not windowed");
                };
                if window.full != full.header.dims || windowed.header.dims.0 >= window.full.0 {
                    panic!(
                        "{label} {case} {kind}: window {window:?} does not sit in the full source"
                    );
                }
                let from_window = renderer
                    .render(&renderer.upload_source(&windowed).unwrap(), &edits, &tile)
                    .unwrap();
                let from_full = renderer
                    .render(&renderer.upload_source(&full).unwrap(), &edits, &tile)
                    .unwrap();
                if (from_window.width, from_window.height) != (from_full.width, from_full.height) {
                    panic!("{label} {case} {kind}: the tile rendered at different dims");
                }
                let delta = mean_abs_delta(&from_window.bytes, &from_full.bytes);
                let worst = from_window
                    .bytes
                    .iter()
                    .zip(&from_full.bytes)
                    .map(|(a, b)| a.abs_diff(*b))
                    .max()
                    .unwrap_or(0);
                if delta > 0.01 || worst > 1 {
                    panic!(
                        "{label} {case} {kind}: the windowed tile drifted {delta:.4}, worst {worst}"
                    );
                }
            }
        }
    }
}
