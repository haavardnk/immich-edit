use super::*;
use crate::decode::dng::tests::{Blob, doubles, floats, gain_map_payload, words};

fn warp_payload(planes: u32, plane: [f64; 6]) -> Vec<u8> {
    let mut payload = words(&[planes]);
    (0..planes).for_each(|_| payload.extend(doubles(&plane)));
    payload.extend(doubles(&[0.5, 0.5]));
    payload
}

#[test]
fn single_plane_warp_applies_to_every_plane() {
    let blob = Blob::default()
        .op(
            1,
            0,
            &warp_payload(1, [0.98, 0.1, -0.2, 0.05, 0.001, -0.002]),
        )
        .bytes();
    let ops = parse(&blob, OpcodeList::Three);
    let [Opcode::WarpRectilinear(warp)] = ops.as_slice() else {
        panic!("expected one warp");
    };
    assert!(!warp.is_split());
    assert_eq!(warp.planes[2].tangential, [0.001, -0.002]);
}

#[test]
fn identity_and_invalid_warps_are_dropped() {
    let blob = Blob::default()
        .op(1, 0, &warp_payload(3, [1.0, 0.0, 0.0, 0.0, 0.0, 0.0]))
        .op(1, 0, &warp_payload(2, [0.9, 0.0, 0.0, 0.0, 0.0, 0.0]))
        .op(1, 0, &warp_payload(1, [f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0]))
        .bytes();
    assert!(parse(&blob, OpcodeList::Three).is_empty());
}

#[test]
fn vignette_reads_coefficients_and_centre() {
    let payload = doubles(&[0.5, -0.25, 0.125, 0.0, 0.0, 0.4, 0.6]);
    let identity = doubles(&[0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
    let blob = Blob::default()
        .op(3, 0, &payload)
        .op(3, 0, &identity)
        .bytes();
    let ops = parse(&blob, OpcodeList::Three);
    let [Opcode::FixVignetteRadial(v)] = ops.as_slice() else {
        panic!("expected one vignette, got {ops:?}");
    };
    assert_eq!(v.k, [0.5, -0.25, 0.125, 0.0, 0.0]);
    assert_eq!(v.center, [0.4, 0.6]);
}

#[test]
fn bad_pixel_list_reads_points_and_rects() {
    let payload = words(&[1, 2, 1, 10, 20, 30, 40, 5, 6, 7, 8]);
    let blob = Blob::default()
        .op(5, 0, &payload)
        .op(4, 0, &words(&[0, 1]))
        .bytes();
    let ops = parse(&blob, OpcodeList::One);
    assert_eq!(
        ops,
        vec![
            Opcode::FixBadPixelsList(BadPixelList {
                points: vec![[10, 20], [30, 40]],
                rects: vec![[5, 6, 7, 8]],
            }),
            Opcode::FixBadPixelsConstant(0),
        ]
    );
}

#[test]
fn hostile_counts_skip_only_their_opcode() {
    let lying = words(&[0, u32::MAX, 0]);
    let huge_map = [
        words(&[0, 0, 10, 10, 0, 1, 1, 1, 0xFFFF, 0xFFFF]),
        doubles(&[0.1, 0.1, 0.0, 0.0]),
        words(&[0xFFFF]),
    ]
    .concat();
    let zero_pitch = [
        words(&[0, 0, 10, 10, 0, 1, 0, 1, 1, 1]),
        doubles(&[1.0, 1.0, 0.0, 0.0]),
        words(&[1]),
        floats(&[2.0]),
    ]
    .concat();
    let infinite = gain_map_payload([0, 0, 4, 4], [0, 1], [1, 1], &[f32::INFINITY]);
    let good = gain_map_payload([0, 0, 4, 4], [0, 1], [1, 1], &[1.5]);
    let blob = Blob::default()
        .op(5, 0, &lying)
        .op(9, 0, &huge_map)
        .op(9, 0, &zero_pitch)
        .op(9, 0, &infinite)
        .op(9, 0, &good)
        .bytes();
    let ops = parse(&blob, OpcodeList::Two);
    assert!(matches!(ops.as_slice(), [Opcode::GainMap(m)] if m.gains == vec![1.5]));
}

#[test]
fn unsupported_opcodes_are_skipped_and_parsing_continues() {
    let good = gain_map_payload([0, 0, 4, 4], [0, 1], [1, 1], &[1.5]);
    let blob = Blob::default()
        .op(7, 1, &[0; 12])
        .op(14, 0, &[0; 40])
        .op(99, 1, &[])
        .op(9, 0, &good)
        .bytes();
    assert_eq!(parse(&blob, OpcodeList::Two).len(), 1);
}

#[test]
fn identity_gain_maps_are_dropped() {
    let identity = gain_map_payload([0, 0, 4, 4], [0, 3], [2, 2], &[1.0; 12]);
    let blob = Blob::default().op(9, 0, &identity).bytes();
    assert!(parse(&blob, OpcodeList::Two).is_empty());
}

#[test]
fn truncated_lists_keep_complete_opcodes() {
    let good = gain_map_payload([0, 0, 4, 4], [0, 1], [1, 1], &[1.5]);
    let blob = Blob::default().op(9, 0, &good).op(9, 0, &good).bytes();
    let cut = blob.len() - 3;
    assert_eq!(parse(&blob[..cut], OpcodeList::Two).len(), 1);
    assert!(parse(&blob[..2], OpcodeList::Two).is_empty());
    let mut inflated = blob.clone();
    inflated[..4].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(parse(&inflated, OpcodeList::Two).len(), 2);
    assert!(parse(&[0, 0, 0, 0], OpcodeList::Two).is_empty());
}

#[test]
fn out_of_range_coefficients_are_rejected() {
    let wild_vignette = doubles(&[1e6, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]);
    let wild_warp = warp_payload(1, [1e30, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let wild_gain = gain_map_payload([0, 0, 4, 4], [0, 1], [1, 1], &[5000.0]);
    let blob = Blob::default()
        .op(3, 0, &wild_vignette)
        .op(1, 0, &wild_warp)
        .op(9, 0, &wild_gain)
        .bytes();
    assert!(parse(&blob, OpcodeList::Three).is_empty());
}
