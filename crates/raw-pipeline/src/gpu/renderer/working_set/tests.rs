use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

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

const SAMPLE_EVERY: Duration = Duration::from_micros(100);
const MAX_OVERESTIMATE: f64 = 1.25;
const DEPTH: &str = "depth";

static MEASURING: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy)]
enum Output {
    Image,
    Source,
}

fn peak_render(
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
    tiles: TilePlan,
    output: Output,
) -> Option<(u64, u64)> {
    let display_tile_edge = tiles.display_edge();
    let _measuring = MEASURING.lock().unwrap_or_else(PoisonError::into_inner);
    let renderer = GpuRenderer::with_options(GpuRendererOptions {
        texture_cache_max_bytes: Some(u64::MAX / 2),
        memory_budget_bytes: Some(u64::MAX),
        display_tile_edge,
        ..GpuRendererOptions::default()
    })
    .ok()?;
    let device = renderer.ctx.device.clone();
    let base = allocated_bytes(&device)?;
    let peak = Arc::new(AtomicU64::new(base));
    let done = Arc::new(AtomicBool::new(false));
    let sampler = {
        let (peak, done, device) = (peak.clone(), done.clone(), device.clone());
        thread::spawn(move || {
            while !done.load(Ordering::Relaxed) {
                peak.fetch_max(allocated_bytes(&device).unwrap_or(0), Ordering::Relaxed);
                thread::sleep(SAMPLE_EVERY);
            }
        })
    };
    let rendered = match output {
        Output::Image => renderer.render(frame, edits, opts).map(drop),
        Output::Source => renderer.render_source(frame, edits, opts, None).map(drop),
    };
    done.store(true, Ordering::Relaxed);
    sampler.join().expect("sampler thread");
    if let Err(e) = rendered {
        panic!("the measured render failed: {e}");
    }
    let estimate = match output {
        Output::Image => renderer.estimate_render(RenderSource::Raw(frame), edits, opts, tiles),
        Output::Source => renderer.estimate_source(frame, edits, opts, tiles),
    };
    Some((peak.load(Ordering::Relaxed) - base, estimate))
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

fn check(name: &str, frame: &RawFrame, edits: &Edits, opts: &RenderOptions, tiles: TilePlan) {
    measure(name, frame, edits, opts, tiles, Output::Image);
}

fn measure(
    name: &str,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
    tiles: TilePlan,
    output: Output,
) {
    let Some((measured, estimate)) = peak_render(frame, edits, opts, tiles, output) else {
        return;
    };
    let ratio = estimate as f64 / measured as f64;
    eprintln!(
        "{name}: measured {} MiB, estimate {} MiB, ratio {ratio:.3}",
        measured >> 20,
        estimate >> 20
    );
    if !(1.0..=MAX_OVERESTIMATE).contains(&ratio) {
        panic!("{name}: estimate {estimate} against a measured peak of {measured} is {ratio:.3}x");
    }
}

#[test]
fn the_estimate_covers_the_measured_peak_of_a_24_mp_export() {
    check(
        "24 MP export",
        &sharp_mosaic(6000, 4000),
        &full_edits(),
        &export_16(),
        TilePlan::SinglePass,
    );
}

#[test]
fn the_estimate_covers_the_measured_peak_of_display_tiles_with_a_context() {
    let mut edits = full_edits();
    edits.retouch.clear();
    check(
        "24 MP display tiles",
        &sharp_mosaic(6000, 4000),
        &edits,
        &export_16(),
        TilePlan::DisplayTiled { edge: 512 },
    );
}

#[test]
fn the_estimate_covers_the_measured_peak_of_a_tile_source() {
    let mut edits = full_edits();
    edits.retouch.clear();
    for tiles in [TilePlan::SinglePass, TilePlan::DisplayTiled { edge: 512 }] {
        measure(
            &format!("24 MP tile source {tiles:?}"),
            &sharp_mosaic(6000, 4000),
            &edits,
            &tile_source(),
            tiles,
            Output::Source,
        );
    }
}

#[test]
#[ignore = "allocates about 28 GiB of gpu memory"]
fn the_estimate_covers_the_measured_peak_of_a_100_mp_export() {
    check(
        "100 MP export",
        &sharp_mosaic(11648, 8736),
        &full_edits(),
        &export_16(),
        TilePlan::SinglePass,
    );
}
