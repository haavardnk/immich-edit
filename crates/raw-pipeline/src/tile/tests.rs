use super::{Rect, SensorChain, SensorTile, Warp, support};
use crate::edits::LensEdits;
use crate::geom::dng_warp::{DngWarp, WarpPlane};

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

fn contains(outer: &Rect, inner: &Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
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
                        if !contains(&bounds, &at) {
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
        let detail = tile.detail();
        let input = tile.input();
        let exact = tile.hist.expand(chain.detail, chain.sensor);
        if !contains(&detail, &exact) {
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
