use super::*;
use crate::decode::mosaic::ActiveArea;

fn map(
    area: [u32; 4],
    plane: [u32; 2],
    pitch: [u32; 2],
    grid: [usize; 2],
    gains: Vec<f32>,
) -> GainMap {
    let map_planes = gains.len() / (grid[0] * grid[1]);
    GainMap {
        area: MapArea {
            top: area[0],
            left: area[1],
            bottom: area[2],
            right: area[3],
        },
        plane: plane[0],
        planes: plane[1],
        pitch,
        points: grid,
        spacing: [
            1.0 / (grid[0] - 1).max(1) as f64,
            1.0 / (grid[1] - 1).max(1) as f64,
        ],
        origin: [0.0, 0.0],
        map_planes,
        gains,
    }
}

fn gains(ops: Vec<(OpcodeList, Gain)>) -> Gains {
    Gains {
        ops: ops
            .into_iter()
            .map(|(list, gain)| StagedGain {
                list,
                after_warp: false,
                gain,
            })
            .collect(),
    }
}

#[test]
fn map_interpolates_bilinearly_and_clamps_outside_its_grid() {
    let m = map(
        [0, 0, 4, 4],
        [0, 1],
        [1, 1],
        [2, 2],
        vec![1.0, 2.0, 3.0, 4.0],
    );
    assert_eq!(m.sample([0.0, 0.0], 0), 1.0);
    assert_eq!(m.sample([1.0, 1.0], 0), 4.0);
    assert_eq!(m.sample([-0.5, 2.0], 0), 2.0);
    assert!((m.sample([0.5, 0.5], 0) - 2.5).abs() < 1e-6);
    assert!((m.sample([0.25, 0.0], 0) - 1.5).abs() < 1e-6);
}

#[test]
fn map_covers_its_half_open_rect_at_its_pitch() {
    let m = map([1, 0, 5, 4], [0, 1], [2, 2], [1, 1], vec![2.0]);
    assert!(m.covers([0, 1]) && m.covers([2, 3]));
    assert!(!m.covers([1, 1]) && !m.covers([0, 2]) && !m.covers([0, 0]));
    assert!(!m.covers([0, 5]) && !m.covers([4, 1]));
}

#[test]
fn map_plane_uses_the_absolute_plane_index() {
    let shared = map([0, 0, 1, 1], [1, 2], [1, 1], [1, 1], vec![2.0]);
    assert_eq!(
        (0..4).map(|p| shared.map_plane(p)).collect::<Vec<_>>(),
        vec![None, Some(0), Some(0), None]
    );
    let split = map([0, 0, 1, 1], [0, 3], [1, 1], [1, 1], vec![2.0, 3.0, 4.0]);
    assert_eq!(split.map_plane(2), Some(2));
}

#[test]
fn radial_gain_is_one_at_the_centre_and_the_polynomial_at_the_corner() {
    let radial = VignetteRadial {
        k: [1.0, 0.5, 0.0, 0.0, 0.0],
        center: [0.5, 0.5],
    };
    let frame = radial.frame([1000.0, 1000.0]);
    assert!((frame.gain([500.5, 500.5]) - 1.0).abs() < 1e-5);
    let corner = frame.gain([999.5, 999.5]);
    assert!((corner - 2.5).abs() < 0.01, "{corner}");
    let off = VignetteRadial {
        k: [1.0, 0.0, 0.0, 0.0, 0.0],
        center: [0.25, 0.5],
    }
    .frame([1000.0, 1000.0]);
    assert!(off.gain([249.5, 499.5]) < 1.0001 && off.gain([999.5, 0.5]) > 1.99);
}

#[test]
fn mosaic_gains_index_list_two_by_phase_and_list_three_by_colour() {
    let phase_r = map([0, 0, 4, 4], [0, 1], [2, 2], [1, 1], vec![2.0]);
    let colour = map([0, 0, 4, 4], [0, 3], [1, 1], [1, 1], vec![3.0, 5.0, 7.0]);
    let g = gains(vec![
        (OpcodeList::Two, Gain::Map(phase_r)),
        (OpcodeList::Three, Gain::Map(colour)),
    ]);
    let mut data = vec![1.0f32; 16];
    g.apply_mosaic(&mut Mosaic::whole(&mut data, 4, 4, "RGGB"), None);
    assert_eq!(&data[..4], &[6.0, 5.0, 6.0, 5.0]);
    assert_eq!(&data[4..8], &[5.0, 7.0, 5.0, 7.0]);
}

#[test]
fn window_gains_match_whole_buffer_gains_at_an_offset() {
    let radial = VignetteRadial {
        k: [0.8, 0.0, 0.0, 0.0, 0.0],
        center: [0.45, 0.55],
    };
    let m = map(
        [0, 0, 6, 8],
        [0, 1],
        [2, 2],
        [2, 2],
        vec![1.0, 1.5, 2.0, 2.5],
    );
    let g = gains(vec![
        (OpcodeList::Three, Gain::Radial(radial)),
        (OpcodeList::Two, Gain::Map(m)),
    ]);
    let mut active = vec![1.0f32; 8 * 6];
    g.apply_mosaic(&mut Mosaic::whole(&mut active, 8, 6, "GBRG"), None);
    let mut full = vec![1.0f32; 11 * 9];
    let area = ActiveArea {
        origin: [3, 2],
        size: [8, 6],
    };
    g.apply_mosaic(&mut Mosaic::window(&mut full, 11, area, "GBRG"), None);
    let window: Vec<f32> = (0..6)
        .flat_map(|y| full[(y + 2) * 11 + 3..(y + 2) * 11 + 11].to_vec())
        .collect();
    assert_eq!(window, active);
    assert_eq!(full[0], 1.0);
    assert_eq!(full[11 * 9 - 1], 1.0);
}

#[test]
fn rgb_gains_use_the_channel_as_plane_for_both_lists() {
    let shared = map([0, 0, 2, 2], [0, 1], [1, 1], [1, 1], vec![2.0]);
    let per_channel = map([0, 0, 2, 2], [0, 3], [1, 1], [1, 1], vec![1.0, 3.0, 5.0]);
    let g = gains(vec![
        (OpcodeList::Two, Gain::Map(shared)),
        (OpcodeList::Three, Gain::Map(per_channel)),
    ]);
    let mut rgb = vec![1.0f32; 2 * 2 * 3];
    g.apply_rgb(&mut rgb, 2, 2, None);
    assert_eq!(&rgb[..3], &[2.0, 3.0, 5.0]);
}

#[test]
fn gains_after_the_warp_are_evaluated_where_the_warp_moves_each_site() {
    use crate::geom::dng_warp::WarpPlane;
    let zoom = WarpPlane {
        radial: [0.5, 0.0, 0.0, 0.0],
        tangential: [0.0, 0.0],
    };
    let warp = DngWarp {
        planes: [zoom; 3],
        center: [0.5, 0.5],
    };
    let ramp = map(
        [0, 0, 1, 64],
        [0, 3],
        [1, 1],
        [1, 2],
        vec![1.0, 1.0, 1.0, 3.0, 3.0, 3.0],
    );
    let mut g = gains(vec![(OpcodeList::Three, Gain::Map(ramp))]);
    let mut before = vec![1.0f32; 64 * 3];
    g.apply_rgb(&mut before, 64, 1, Some(&warp));
    g.ops[0].after_warp = true;
    let mut after = vec![1.0f32; 64 * 3];
    g.apply_rgb(&mut after, 64, 1, Some(&warp));
    let site = 40 * 3;
    let destination = 32.0 + (40.5 - 32.0) / 0.5;
    let expected = 1.0 + 2.0 * (destination / 64.0) as f32;
    assert!(
        (after[site] - expected).abs() < 1e-3,
        "{} vs {expected}",
        after[site]
    );
    assert!((before[site] - (1.0 + 2.0 * 40.5 / 64.0)).abs() < 1e-3);
}

#[test]
fn stacked_gains_stay_finite() {
    let huge = map([0, 0, 2, 2], [0, 1], [1, 1], [1, 1], vec![1000.0]);
    let zero = map([0, 0, 2, 2], [0, 1], [1, 1], [1, 1], vec![0.0]);
    let mut ops: Vec<(OpcodeList, Gain)> = (0..40)
        .map(|_| (OpcodeList::Two, Gain::Map(huge.clone())))
        .collect();
    ops.push((OpcodeList::Two, Gain::Map(zero)));
    let mut rgb = vec![1.0f32; 2 * 2 * 3];
    gains(ops).apply_rgb(&mut rgb, 2, 2, None);
    assert!(rgb[0].is_finite() && rgb[0] == 0.0);
}
