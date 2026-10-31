use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

use raw_pipeline_testkit::memory::{check, finish};

use super::super::test_frame::checker_frame;
use super::super::{GpuRenderer, GpuRendererOptions};
use crate::edits::{
    CropRect, Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer, MaskSource,
    MaskedEdits, RetouchMode, RetouchStroke, Vec2f,
};
use crate::frame::{BitDepth, OutputFormat, PngCompression, RawFrame, RenderOptions};
use crate::gpu::memory::allocated_bytes;
use crate::gpu::source::RenderSource;
use crate::mask_raster::{MaskRaster, RasterMap};
use crate::tile::TilePlan;

const DEPTH: &str = "depth";
const SAMPLE_EVERY: Duration = Duration::from_micros(100);
const DISPLAY_TILES: TilePlan = TilePlan::DisplayTiled {
    edge: 2048,
    held: false,
};
const HELD_TILES: TilePlan = TilePlan::DisplayTiled {
    edge: 2048,
    held: true,
};

#[derive(Clone, Copy)]
enum Output {
    Image,
    Source,
}

struct Case {
    name: &'static str,
    edits: Edits,
    options: RenderOptions,
    tiles: TilePlan,
    output: Output,
}

fn sampled_peak(sample: impl Fn() -> u64 + Sync, work: impl FnOnce()) -> u64 {
    let base = sample();
    let peak = AtomicU64::new(base);
    let done = AtomicBool::new(false);
    thread::scope(|scope| {
        scope.spawn(|| {
            while !done.load(Ordering::Relaxed) {
                peak.fetch_max(sample(), Ordering::Relaxed);
                thread::sleep(SAMPLE_EVERY);
            }
        });
        work();
        done.store(true, Ordering::Relaxed);
    });
    peak.load(Ordering::SeqCst).saturating_sub(base)
}

fn measure(frame: &RawFrame, case: &Case, failures: &mut Vec<String>) {
    let Ok(renderer) = GpuRenderer::with_options(GpuRendererOptions {
        texture_cache_max_bytes: Some(u64::MAX / 2),
        memory_budget_bytes: Some(u64::MAX),
        tiles: Some(case.tiles),
        ..GpuRendererOptions::default()
    }) else {
        return;
    };
    let device = renderer.ctx.device.clone();
    if allocated_bytes(&device).is_none() {
        return;
    }
    let (edits, opts) = (&case.edits, &case.options);
    let measured = sampled_peak(
        || allocated_bytes(&device).unwrap_or(0),
        || {
            let rendered = match case.output {
                Output::Image => renderer.render(frame, edits, opts).map(drop),
                Output::Source => renderer.render_source(frame, edits, opts, None).map(drop),
            };
            if let Err(e) = rendered {
                failures.push(format!("{}: the measured render failed: {e}", case.name));
            }
        },
    );
    let estimate = match case.output {
        Output::Image => {
            renderer.estimate_render(RenderSource::Raw(frame), edits, opts, case.tiles)
        }
        Output::Source => renderer.estimate_source(frame, edits, opts, case.tiles),
    };
    check(case.name, measured, estimate.device, failures);
}

fn full_edits() -> Edits {
    let mut edits = Edits::default();
    edits.detail.luma_nr_amount = 40.0;
    edits.detail.color_nr_amount = 30.0;
    edits.detail.sharpen_amount = Some(60.0);
    edits.basic.dehaze = 30.0;
    edits.basic.clarity = 25.0;
    edits.basic.texture = 20.0;
    edits.tone.shadows = 30.0;
    edits.lens_blur.amount = 40.0;
    edits.lens_blur.depth_raster_id = DEPTH.into();
    edits.retouch = vec![RetouchStroke {
        id: "heal".into(),
        mode: RetouchMode::Heal,
        points: vec![Vec2f { x: 0.3, y: 0.4 }, Vec2f { x: 0.35, y: 0.45 }],
        radius: 0.05,
        hardness: 0.5,
        opacity: 1.0,
        source: Vec2f { x: 0.6, y: 0.4 },
        enabled: true,
    }];
    edits.masks = vec![MaskLayer {
        id: "highlights".into(),
        name: String::new(),
        enabled: true,
        amount: 1.0,
        invert: false,
        components: vec![MaskComponent {
            id: "luma".into(),
            enabled: true,
            mode: MaskComponentMode::Add,
            invert: false,
            kind: MaskComponentKind::LumaRange {
                min: 0.5,
                max: 1.0,
                softness: 0.2,
            },
            source: MaskSource::Manual,
            generated: None,
        }],
        edits: MaskedEdits {
            exposure_ev: Some(-0.5),
            ..Default::default()
        },
    }];
    edits
}

fn sharp_mosaic(width: usize, height: usize) -> RawFrame {
    let data = (0..width * height)
        .map(|i| {
            let (x, y) = (i % width, i / width);
            let hash = (x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663)) % 1000;
            0.2 + 0.5 * (x as f32 / width as f32) + hash as f32 / 20_000.0
        })
        .collect();
    let mut frame = checker_frame();
    frame.meta.width = width;
    frame.meta.height = height;
    frame.meta.is_raw = true;
    frame.meta.capture_sigma = Some(0.8);
    frame.cfa_pattern = "RGGB".into();
    frame.cpp = 1;
    frame.data = data;
    frame
}

fn depth_raster() -> RasterMap {
    let (w, h) = (256, 171);
    let bytes = (0..w * h).map(|i| (i % w * 255 / w) as u8).collect();
    let mut rasters = RasterMap::new();
    rasters.insert(
        DEPTH.into(),
        Arc::new(MaskRaster::new(w, h, bytes).unwrap()),
    );
    rasters
}

fn export_16() -> RenderOptions {
    RenderOptions {
        rasters: depth_raster(),
        max_edge: u32::MAX,
        quality: true,
        output: OutputFormat::Png {
            bit_depth: BitDepth::Sixteen,
            compression: PngCompression::Fast,
        },
        ..RenderOptions::default()
    }
}

fn tile_source() -> RenderOptions {
    RenderOptions {
        rasters: depth_raster(),
        max_edge: u32::MAX,
        roi: Some(CropRect {
            x: 0.55,
            y: 0.4,
            w: 0.3,
            h: 0.3,
        }),
        ..RenderOptions::default()
    }
}

fn without_retouch() -> Edits {
    Edits {
        retouch: Vec::new(),
        ..full_edits()
    }
}

fn case(name: &'static str, edits: Edits, options: RenderOptions, tiles: TilePlan) -> Case {
    Case {
        name,
        edits,
        options,
        tiles,
        output: Output::Image,
    }
}

fn source(name: &'static str, tiles: TilePlan) -> Case {
    Case {
        output: Output::Source,
        ..case(name, without_retouch(), tile_source(), tiles)
    }
}

fn require_covered(frame: &RawFrame, cases: &[Case]) {
    let mut failures = Vec::new();
    for case in cases {
        measure(frame, case, &mut failures);
    }
    finish(failures);
}

#[test]
fn the_estimate_covers_the_measured_peak_of_24_mp_renders() {
    require_covered(
        &sharp_mosaic(6000, 4000),
        &[
            case("export", full_edits(), export_16(), TilePlan::SinglePass),
            case(
                "display tiles",
                without_retouch(),
                export_16(),
                DISPLAY_TILES,
            ),
            case("held display tiles", full_edits(), export_16(), HELD_TILES),
            source("tile source", TilePlan::SinglePass),
            source("display-tiled tile source", DISPLAY_TILES),
        ],
    );
}

#[test]
#[ignore = "allocates about 28 GiB of gpu memory"]
fn the_estimate_covers_the_measured_peak_of_a_100_mp_export() {
    require_covered(
        &sharp_mosaic(11648, 8736),
        &[case(
            "100 MP export",
            full_edits(),
            export_16(),
            TilePlan::SinglePass,
        )],
    );
}
