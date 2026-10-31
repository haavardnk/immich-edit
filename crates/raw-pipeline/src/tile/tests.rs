use super::retouch::closure;
use super::{Rect, SensorChain, SensorTile, StrokeReach, Warp, support};
use crate::edits::{LensEdits, RetouchMode, RetouchStroke, Vec2f};
use crate::geom::dng_warp::{DngWarp, WarpPlane};
use crate::ops::retouch::stroke_geometry;

const XTRANS: &str = "GGRGGBGGBGGRBRGRBGGGBGGRGGRGGBRBGBRG";

fn covered(rects: impl Iterator<Item = Rect>, (w, h): (u32, u32)) -> Vec<u32> {
    let mut hits = vec![0u32; (w * h) as usize];
    for rect in rects {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                hits[(y * w + x) as usize] += 1;
            }
        }
    }
    hits
}

fn partitions(rects: impl Iterator<Item = Rect>, dims: (u32, u32)) -> bool {
    covered(rects, dims).iter().all(|&n| n == 1)
}

fn lens() -> LensEdits {
    LensEdits {
        profile_enabled: Some(true),
        ca_enabled: true,
        distortion_amount: 100.0,
        k1: -0.12,
        k2: 0.03,
        ca_red_scale_x10000: 8.0,
        ca_blue_scale_x10000: -6.0,
        ..LensEdits::default()
    }
}

fn dng() -> DngWarp {
    let plane = |k1: f32| WarpPlane {
        radial: [1.0, k1, 0.01, 0.0],
        tangential: [0.002, -0.001],
    };
    DngWarp {
        planes: [plane(0.04), plane(0.05), plane(0.06)],
        center: [0.47, 0.53],
    }
}

fn chain(sensor: (u32, u32), working: (u32, u32), warps: Vec<Warp>) -> SensorChain {
    SensorChain {
        sensor,
        working,
        align: support::alignment(XTRANS, 1, 1),
        demosaic: support::demosaic(XTRANS, 1, 1),
        warps,
        detail: 40,
        strokes: Vec::new(),
    }
}

#[test]
fn a_grid_partitions_the_frame_for_any_edge() {
    for dims in [(1, 1), (100, 37), (64, 64), (130, 257)] {
        for edge in [1, 7, 64, 100, 4096] {
            if !partitions(Rect::grid(dims, edge), dims) {
                panic!("grid of {dims:?} at edge {edge} does not partition it");
            }
        }
    }
}

#[test]
fn expanding_and_aligning_stay_inside_the_frame() {
    let frame = (100, 60);
    let cases = [
        (
            Rect::bounds(10, 10, 20, 20),
            5,
            4,
            Rect::bounds(4, 4, 25, 25),
        ),
        (Rect::bounds(0, 0, 10, 10), 7, 4, Rect::bounds(0, 0, 17, 17)),
        (
            Rect::bounds(90, 50, 100, 60),
            30,
            12,
            Rect::bounds(60, 12, 100, 60),
        ),
    ];
    for (rect, margin, align, want) in cases {
        let got = rect.expand(margin, frame).align(align);
        if got != want {
            panic!("{rect:?} + {margin} aligned to {align}: {got:?}, expected {want:?}");
        }
    }
}

#[test]
fn alignment_covers_the_cfa_period_and_the_noise_histogram_grid() {
    let cases = [
        ("RGGB", 1, 1, 4, 2),
        ("RGGB", 1, 2, 4, 0),
        (XTRANS, 1, 1, 12, 3),
        (XTRANS, 1, 3, 4, 0),
        (XTRANS, 1, 6, 4, 0),
        ("", 3, 1, 4, 0),
    ];
    for (cfa, cpp, block, align, demosaic) in cases {
        let got = (
            support::alignment(cfa, cpp, block),
            support::demosaic(cfa, cpp, block),
        );
        if got != (align, demosaic) {
            panic!("{cfa:?} cpp {cpp} block {block}: {got:?}, expected ({align}, {demosaic})");
        }
    }
}

#[test]
fn the_lanczos_footprint_holds_every_tap_the_resample_shader_reads() {
    for (src, dst) in [(1203, 401), (1000, 997), (4000, 1024), (37, 5)] {
        let scale = src as f32 / dst as f32;
        let support = 3.0 * scale.max(1.0);
        for core in Rect::grid((dst, 1), 13).map(|r| (r.x, r.right())) {
            let (lo, hi) = support::lanczos(core, scale, src);
            for idx in core.0..core.1 {
                let center = (idx as f32 + 0.5) * scale;
                let first = ((center - support - 0.5).floor() as i64).max(0);
                let last = ((center + support - 0.5).ceil() as i64).min(i64::from(src) - 1);
                if first < i64::from(lo) || last >= i64::from(hi) {
                    panic!("{src}->{dst} output {idx} reads {first}..={last} outside {lo}..{hi}");
                }
            }
        }
    }
}

#[test]
fn warp_bounds_hold_every_bicubic_tap_of_the_rect() {
    let frame = (600, 400);
    let tap = |s: f32, d: f32, n: u32| (s.floor() + d).clamp(0.0, n as f32 - 1.0) as u32;
    for warp in [Warp::Lens(lens()), Warp::Dng(dng())] {
        let maps = warp.maps(frame);
        for rect in Rect::grid(frame, 97) {
            let bounds = warp.source(rect, frame);
            for (x, y) in
                (rect.y..rect.bottom()).flat_map(|y| (rect.x..rect.right()).map(move |x| (x, y)))
            {
                for [sx, sy] in maps.iter().map(|map| map(x as f32, y as f32)) {
                    for d in [-1.0, 2.0] {
                        let at = Rect {
                            x: tap(sx, d, frame.0),
                            y: tap(sy, d, frame.1),
                            w: 1,
                            h: 1,
                        };
                        if !bounds.contains(at) {
                            panic!(
                                "{warp:?} pixel ({x}, {y}) of {rect:?} taps {at:?} outside {bounds:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn a_warp_that_samples_only_outside_the_frame_bounds_the_clamped_edge_tap() {
    let frame = (600, 400);
    let rect = Rect::bounds(0, 0, 50, 50);
    let cases = [
        (10_000.0, -10_000.0, Rect::bounds(599, 0, 600, 1)),
        (-10_000.0, 10_000.0, Rect::bounds(0, 399, 1, 400)),
    ];
    for (dx, dy, edge) in cases {
        let bounds = support::warp(rect, frame, |x, y| [x + dx, y + dy]);
        if bounds != edge {
            panic!("shift ({dx}, {dy}) bounds {bounds:?}, expected {edge:?}");
        }
    }
}

fn check_tiles(chain: &SensorChain, tiles: &[SensorTile]) {
    if !partitions(tiles.iter().map(|t| t.core), chain.working) {
        panic!("tile cores do not partition the working image");
    }
    if !partitions(tiles.iter().map(|t| t.hist), chain.sensor) {
        panic!("noise histogram cells do not partition the sensor");
    }
    for tile in tiles {
        let detail = tile.detail;
        let input = tile.input();
        let exact = tile.hist.expand(chain.detail, chain.sensor);
        if !detail.contains(exact) {
            panic!("detail {detail:?} misses the histogram margin {exact:?}");
        }
        if detail.x % chain.align != 0 || detail.y % chain.align != 0 {
            panic!("detail {detail:?} is not aligned to {}", chain.align);
        }
        if input.x % chain.align != 0 || input.y % chain.align != 0 {
            panic!("input {input:?} is not aligned to {}", chain.align);
        }
        if tile.hist.x % support::HIST_ALIGN != 0 || tile.hist.y % support::HIST_ALIGN != 0 {
            panic!("histogram cell {:?} is off the sample grid", tile.hist);
        }
    }
}

#[test]
fn sensor_tiles_partition_cores_and_histogram_cells_with_aligned_margins() {
    let cases = [
        chain((1203, 805), (1203, 805), Vec::new()),
        chain((1203, 805), (401, 268), Vec::new()),
        chain(
            (1203, 805),
            (1203, 805),
            vec![Warp::Dng(dng()), Warp::Lens(lens())],
        ),
        chain((1203, 805), (602, 403), vec![Warp::Lens(lens())]),
    ];
    for chain in &cases {
        for edge in [16, 100, 256] {
            check_tiles(chain, &chain.tiles(edge));
        }
        let whole = chain.whole();
        if whole.rects.iter().any(|r| *r != Rect::full(chain.sensor))
            || whole.core != Rect::full(chain.working)
        {
            panic!("the whole-frame tile is not the full frame: {whole:?}");
        }
    }
}

fn heal(points: &[(f32, f32)], radius: f32, source: (f32, f32)) -> RetouchStroke {
    RetouchStroke {
        id: String::new(),
        mode: RetouchMode::Heal,
        points: points.iter().map(|&(x, y)| Vec2f { x, y }).collect(),
        radius,
        hardness: 0.5,
        opacity: 1.0,
        source: Vec2f {
            x: source.0,
            y: source.1,
        },
        enabled: true,
    }
}

#[test]
fn a_stroke_reach_holds_every_tap_the_retouch_prep_reads() {
    let frame = (600, 400);
    let strokes = [
        heal(&[(0.3, 0.4), (0.5, 0.45)], 0.05, (0.6, 0.7)),
        heal(&[(0.9, 0.1)], 0.04, (0.999, 0.001)),
        heal(&[(0.7, 0.6)], 0.03, (0.2137, 0.3311)),
    ];
    let tap = |s: f32, d: f32, n: u32| (s.floor() + d).clamp(0.0, n as f32 - 1.0) as u32;
    for orientation in [(false, false, false), (true, true, false)] {
        for stroke in &strokes {
            let geom = stroke_geometry(stroke, 600, 400, orientation).unwrap();
            let reach = StrokeReach::new(0, &geom, frame);
            let b = &geom.bbox;
            for (x, y) in (b.y0..b.y1).flat_map(|y| (b.x0..b.x1).map(move |x| (x, y))) {
                let (sx, sy) = (x as f32 + geom.off_x, y as f32 + geom.off_y);
                for d in [-1.0, 2.0] {
                    let at = Rect::bounds(
                        tap(sx, d, frame.0),
                        tap(sy, d, frame.1),
                        tap(sx, d, frame.0) + 1,
                        tap(sy, d, frame.1) + 1,
                    );
                    if !reach.read.contains(at) {
                        panic!("{orientation:?} ({x}, {y}) taps {at:?} outside {reach:?}");
                    }
                }
            }
        }
    }
}

#[test]
fn a_stroke_that_reaches_a_tile_reads_only_demosaic_exact_pixels() {
    let mut chain = chain((1203, 805), (1203, 805), Vec::new());
    let core = Rect::bounds(0, 0, 256, 256);
    let edge = chain.tile(core).detail.right();
    let stroke = StrokeReach {
        index: 0,
        write: Rect::bounds(edge - 20, 100, edge - 10, 110),
        read: Rect::bounds(edge - 20, 100, edge - 1, 110),
    };
    chain.strokes = vec![stroke];
    let tile = chain.tile(core);
    let input = tile.input();
    let exact = Rect::bounds(
        input.x,
        input.y,
        input.right() - chain.demosaic,
        input.bottom(),
    );
    if tile.strokes != [0] || !exact.contains(stroke.read) {
        panic!(
            "{stroke:?} runs as {:?} outside the demosaic-exact {exact:?}",
            tile.strokes
        );
    }
}

#[test]
fn the_retouch_closure_follows_earlier_strokes_under_a_source_in_stroke_order() {
    let reaches = [
        (Rect::bounds(50, 50, 60, 60), Rect::bounds(80, 80, 90, 90)),
        (
            Rect::bounds(100, 0, 110, 10),
            Rect::bounds(100, 20, 110, 30),
        ),
        (Rect::bounds(5, 5, 15, 15), Rect::bounds(55, 55, 65, 65)),
        (
            Rect::bounds(60, 60, 70, 70),
            Rect::bounds(120, 120, 130, 130),
        ),
        (Rect::bounds(200, 200, 210, 210), Rect::bounds(0, 0, 5, 5)),
    ];
    let strokes: Vec<StrokeReach> = reaches
        .iter()
        .enumerate()
        .map(|(index, &(write, source))| StrokeReach {
            index,
            write,
            read: write.union(source),
        })
        .collect();
    let detail = Rect::bounds(0, 0, 10, 10);
    let got = closure(&strokes, detail);
    if got != (Rect::bounds(0, 0, 90, 90), vec![0, 2]) {
        panic!("closure {got:?}");
    }
    if closure(&strokes[3..], detail) != (detail, Vec::new()) {
        panic!("strokes that cannot reach the detail rect joined the closure");
    }
}
