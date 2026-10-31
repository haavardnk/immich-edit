mod display_tiles;
mod sensor_tiles;

use raw_pipeline::decode;
use raw_pipeline::edits::{Edits, MaskComponentKind, RetouchMode, RetouchStroke, Vec2f};
use raw_pipeline::frame::{RawFrame, RenderOptions, RenderedImage};
use raw_pipeline::{GpuRenderer, GpuRendererOptions, TilePlan, TilingReport};
use raw_pipeline_testkit::fixtures::fixture_path;
use raw_pipeline_testkit::gpu::{try_renderer, try_renderer_with};
use raw_pipeline_testkit::retouch::stroke;

const XT2: &str = "Fujifilm_X-T2_14bit_14bit_compressed_3-2.raf";
const CANON_5D: &str = "Canon_EOS_5D_3-2.cr2";

fn decoded(name: &str) -> RawFrame {
    let bytes = std::fs::read(fixture_path(name)).unwrap();
    decode::decode(&bytes).unwrap()
}

struct Pair {
    single: GpuRenderer,
    tiled: GpuRenderer,
    sensor: bool,
}

impl Pair {
    fn new(options: GpuRendererOptions, sensor: bool) -> Option<Self> {
        Some(Self {
            single: try_renderer()?,
            tiled: try_renderer_with(options)?,
            sensor,
        })
    }

    fn forced(tiles: TilePlan) -> Option<Self> {
        let sensor = matches!(tiles, TilePlan::SensorTiled { .. });
        let options = GpuRendererOptions {
            tiles: Some(tiles),
            ..GpuRendererOptions::default()
        };
        Self::new(options, sensor)
    }

    fn budgeted(bytes: u64) -> Option<Self> {
        let options = GpuRendererOptions {
            memory_budget_bytes: Some(bytes),
            ..GpuRendererOptions::default()
        };
        Self::new(options, false)
    }

    fn require(
        &self,
        label: &str,
        frame: &RawFrame,
        edits: &Edits,
        opts: &RenderOptions,
    ) -> RenderedImage {
        let reference = render(&self.single, frame, edits, opts);
        let got = render(&self.tiled, frame, edits, opts);
        require_identical(label, &reference, &got);
        require_tiles(label, got.tiling, self.sensor);
        got
    }

    fn require_modes(&self, name: &str, frame: &RawFrame, edits: &Edits, modes: &[Mode]) {
        for (mode, opts) in modes {
            self.require(&format!("{name} {mode}"), frame, edits, opts);
        }
    }
}

type Mode = (&'static str, RenderOptions);

fn render(
    renderer: &GpuRenderer,
    frame: &RawFrame,
    edits: &Edits,
    opts: &RenderOptions,
) -> RenderedImage {
    match renderer.render(frame, edits, opts) {
        Ok(img) => img,
        Err(e) => panic!("render failed: {e}"),
    }
}

fn require_tiles(label: &str, tiling: TilingReport, sensor: bool) {
    let planned = match tiling.plan {
        TilePlan::SensorTiled { .. } => sensor,
        TilePlan::DisplayTiled { .. } => !sensor,
        TilePlan::SinglePass => false,
    };
    if !planned || tiling.tiles < 4 {
        panic!("{label} rendered as {tiling:?}");
    }
}

fn require_identical(label: &str, single: &RenderedImage, tiled: &RenderedImage) {
    if (single.width, single.height) != (tiled.width, tiled.height) {
        panic!(
            "{label}: tiled {}x{} against single {}x{}",
            tiled.width, tiled.height, single.width, single.height
        );
    }
    let differing = single
        .bytes
        .iter()
        .zip(&tiled.bytes)
        .filter(|(a, b)| a != b)
        .count();
    if differing > 0 || single.bytes.len() != tiled.bytes.len() {
        panic!("{label}: {differing} tiled bytes differ from the single pass");
    }
    if single.histogram != tiled.histogram || single.linear_histogram != tiled.linear_histogram {
        panic!("{label}: tiled histograms differ from the single pass");
    }
    if single.scopes != tiled.scopes {
        panic!("{label}: tiled scopes differ from the single pass");
    }
}

fn radial(center: (f32, f32), radius: (f32, f32)) -> MaskComponentKind {
    MaskComponentKind::Radial {
        center: Vec2f {
            x: center.0,
            y: center.1,
        },
        radius_xy: Vec2f {
            x: radius.0,
            y: radius.1,
        },
        feather: 0.4,
        angle: 0.0,
    }
}

fn strokes() -> Vec<RetouchStroke> {
    let across = [(0.3, 0.33), (0.4, 0.37), (0.48, 0.36)];
    let clone = [(0.66, 0.67), (0.71, 0.72)];
    let (on_the_heal, past_the_corner) = ((0.4, 0.36), (0.99, 0.01));
    vec![
        stroke(RetouchMode::Heal, &across, 0.03, (0.42, 0.55)),
        stroke(RetouchMode::Clone, &clone, 0.03, on_the_heal),
        stroke(RetouchMode::Heal, &[(0.85, 0.15)], 0.02, past_the_corner),
    ]
}
