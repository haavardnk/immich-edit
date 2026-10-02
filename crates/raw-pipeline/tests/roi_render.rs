use raw_pipeline::{
    GpuRenderer, cpu, decode,
    edits::{
        CropRect, Edits, MaskComponent, MaskComponentKind, MaskComponentMode, MaskLayer,
        MaskSource, MaskedEdits,
    },
    frame::{OutputFormat, RawFrame, RenderOptions},
};
use raw_pipeline_testkit::color::luma;
use raw_pipeline_testkit::fixtures::first_decodable_fixture;
use raw_pipeline_testkit::frames::rgb_frame;

fn checker_frame(w: usize, h: usize) -> RawFrame {
    let mut data = vec![0.0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 3;
            let checker = if (x / 3 + y / 3) % 2 == 0 {
                0.12
            } else {
                -0.12
            };
            let base = 0.35 + 0.4 * (x as f32 / (w - 1) as f32);
            data[i] = (base + checker).clamp(0.02, 0.98);
            data[i + 1] = (base * 0.9 + checker).clamp(0.02, 0.98);
            data[i + 2] = (0.9 - base + checker).clamp(0.02, 0.98);
        }
    }
    rgb_frame(w, h, data)
}

fn render_rgb(
    frame: &RawFrame,
    edits: &Edits,
    max_edge: u32,
    roi: Option<CropRect>,
) -> (Vec<u8>, usize, usize) {
    let opts = RenderOptions {
        max_edge,
        roi,
        output: OutputFormat::Rgb8,
        ..Default::default()
    };
    let out = cpu::render(frame, edits, &opts).unwrap();
    (out.bytes, out.width as usize, out.height as usize)
}

fn mean_abs_luma_delta(a: &[u8], b: &[u8]) -> f64 {
    let n = (a.len() / 3) as f64;
    a.chunks_exact(3)
        .zip(b.chunks_exact(3))
        .map(|(p, q)| (luma(p) - luma(q)).abs())
        .sum::<f64>()
        / n
}

fn crop_region(rgb: &[u8], w: usize, x0: usize, y0: usize, cw: usize, ch: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(cw * ch * 3);
    for y in 0..ch {
        let start = ((y0 + y) * w + x0) * 3;
        out.extend_from_slice(&rgb[start..start + cw * 3]);
    }
    out
}

fn tile_matches_full_crop(
    render: impl Fn(&Edits, u32, Option<CropRect>) -> (Vec<u8>, usize, usize),
    roi: CropRect,
    cases: &[(&str, Edits)],
    tolerance: f64,
) {
    for (name, edits) in cases {
        let (full, fw, fh) = render(edits, 4096, None);
        let tile_edge = ((fw as f64 * roi.w as f64).max(fh as f64 * roi.h as f64)).round() as u32;
        let (tile, tw, th) = render(edits, tile_edge, Some(roi));
        let x0 = (roi.x as f64 * fw as f64).round() as usize;
        let y0 = (roi.y as f64 * fh as f64).round() as usize;
        if x0 + tw > fw || y0 + th > fh {
            panic!("{name}: tile {tw}x{th} at {x0},{y0} does not fit in {fw}x{fh}");
        }
        let region = crop_region(&full, fw, x0, y0, tw, th);
        let delta = mean_abs_luma_delta(&tile, &region);
        if delta > tolerance {
            panic!("{name}: roi tile differs from full render crop by {delta}");
        }
    }
}

fn roi_cases() -> Vec<(&'static str, Edits)> {
    let mut vignetted = Edits::default();
    vignetted.effects.vignette_amount = -80.0;
    let mut grained = Edits::default();
    grained.effects.grain_amount = 70.0;
    let mut cropped = Edits::default();
    cropped.geometry.rotate = 90;
    cropped.geometry.crop = Some(CropRect {
        x: 0.1,
        y: 0.2,
        w: 0.6,
        h: 0.5,
    });
    vec![
        ("identity", Edits::default()),
        ("vignette", vignetted),
        ("grain", grained),
        ("rotate_crop", cropped),
    ]
}

fn luma_range_edits() -> Edits {
    Edits {
        masks: vec![MaskLayer {
            id: "L1".into(),
            name: String::new(),
            enabled: true,
            color: "#fff".into(),
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
                    softness: 0.002,
                },
                source: MaskSource::Manual,
                generated: None,
            }],
            edits: MaskedEdits {
                exposure_ev: Some(1.5),
                ..Default::default()
            },
        }],
        ..Default::default()
    }
}

const ROI: CropRect = CropRect {
    x: 0.25,
    y: 0.25,
    w: 0.5,
    h: 0.5,
};

#[test]
fn roi_tile_matches_full_render_crop() {
    let frame = checker_frame(240, 180);
    tile_matches_full_crop(
        |edits, max_edge, roi| render_rgb(&frame, edits, max_edge, roi),
        ROI,
        &roi_cases(),
        0.6,
    );
}

#[test]
fn gpu_roi_tile_matches_full_render_crop() {
    let renderer = match GpuRenderer::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            return;
        }
    };
    let frame = checker_frame(240, 180);
    let render = |edits: &Edits, max_edge: u32, roi: Option<CropRect>| {
        let opts = RenderOptions {
            max_edge,
            roi,
            output: OutputFormat::Rgb8,
            ..Default::default()
        };
        let out = renderer.render(&frame, edits, &opts).unwrap();
        (out.bytes, out.width as usize, out.height as usize)
    };
    tile_matches_full_crop(render, ROI, &roi_cases(), 1.5);
}

#[test]
fn gpu_roi_range_mask_matches_full_render_at_tile_edges() {
    let renderer = match GpuRenderer::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no gpu adapter, skipping: {e}");
            return;
        }
    };
    let w = 2048;
    let h = 1024;
    let data = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            let v =
                0.1 + 0.3 * x as f32 / (w - 1) as f32 + [0.012, -0.006, -0.006][(x + 2 * y) % 3];
            [v, v, v]
        })
        .collect();
    let frame = rgb_frame(w, h, data);
    let edits = luma_range_edits();
    let render = |max_edge: u32, roi: Option<CropRect>| {
        let opts = RenderOptions {
            max_edge,
            roi,
            output: OutputFormat::Rgb8,
            ..Default::default()
        };
        renderer.render(&frame, &edits, &opts).unwrap()
    };
    let full = render(4096, None);
    let tile = render(w as u32 / 2, Some(ROI));
    let fw = full.width as usize;
    let x0 = (ROI.x as f64 * fw as f64).round() as usize;
    let y0 = (ROI.y as f64 * full.height as f64).round() as usize;
    let region = crop_region(
        &full.bytes,
        fw,
        x0,
        y0,
        tile.width as usize,
        tile.height as usize,
    );
    let worst = tile
        .bytes
        .chunks_exact(3)
        .zip(region.chunks_exact(3))
        .map(|(p, q)| (luma(p) - luma(q)).abs())
        .fold(0.0, f64::max);
    if worst > 3.0 {
        panic!("range mask tile edge differs from the full render by {worst}");
    }
}

#[test]
fn capture_sharpen_is_visible_at_one_to_one() {
    let Some(path) = first_decodable_fixture() else {
        eprintln!("no raw fixture; skipping");
        return;
    };
    let bytes = std::fs::read(&path).unwrap();
    let frame = decode::decode(&bytes).unwrap();
    if frame.meta.capture_sigma.is_none() {
        eprintln!("fixture has no capture sigma; skipping");
        return;
    }
    let mut off = Edits::default();
    off.detail.capture_sharpen = false;
    let on = Edits::default();

    let fit_edge = 1024;
    let (fit_on, _, _) = render_rgb(&frame, &on, fit_edge, None);
    let (fit_off, _, _) = render_rgb(&frame, &off, fit_edge, None);
    let fit_delta = mean_abs_luma_delta(&fit_on, &fit_off);

    let roi = CropRect {
        x: 0.375,
        y: 0.375,
        w: 0.25,
        h: 0.25,
    };
    let tile_edge = fit_edge;
    let (roi_on, _, _) = render_rgb(&frame, &on, tile_edge, Some(roi));
    let (roi_off, _, _) = render_rgb(&frame, &off, tile_edge, Some(roi));
    let roi_delta = mean_abs_luma_delta(&roi_on, &roi_off);

    if roi_delta <= fit_delta * 1.5 {
        panic!("capture sharpen delta at 1:1 ({roi_delta}) should exceed fit delta ({fit_delta})");
    }
}
