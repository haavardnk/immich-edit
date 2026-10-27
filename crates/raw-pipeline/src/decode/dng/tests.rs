use super::*;
use crate::geom::dng_warp::WarpPlane;
use raw_pipeline_testkit::fixtures::fixture_path;

#[derive(Default)]
pub(super) struct Blob {
    count: u32,
    body: Vec<u8>,
}

impl Blob {
    pub(super) fn op(mut self, id: u32, flags: u32, payload: &[u8]) -> Self {
        self.count += 1;
        for word in [id, 0x0103_0000, flags, payload.len() as u32] {
            self.body.extend_from_slice(&word.to_be_bytes());
        }
        self.body.extend_from_slice(payload);
        self
    }

    pub(super) fn bytes(self) -> Vec<u8> {
        let mut out = self.count.to_be_bytes().to_vec();
        out.extend(self.body);
        out
    }
}

pub(super) fn words(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

pub(super) fn doubles(values: &[f64]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

pub(super) fn floats(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}

pub(super) fn gain_map_payload(
    area: [u32; 4],
    plane: [u32; 2],
    grid: [u32; 2],
    gains: &[f32],
) -> Vec<u8> {
    let map_planes = gains.len() as u32 / (grid[0] * grid[1]);
    let spacing_v = 1.0 / f64::from((grid[0] - 1).max(1));
    let spacing_h = 1.0 / f64::from((grid[1] - 1).max(1));
    [
        words(&area),
        words(&[plane[0], plane[1], 1, 1, grid[0], grid[1]]),
        doubles(&[spacing_v, spacing_h, 0.0, 0.0]),
        words(&[map_planes]),
        floats(gains),
    ]
    .concat()
}

fn corrections(name: &str) -> DngCorrections {
    let bytes = std::fs::read(fixture_path(name)).unwrap();
    DngCorrections::read(&bytes)
}

fn maps(c: &DngCorrections) -> Vec<(OpcodeList, &gain::GainMap)> {
    c.gains
        .ops
        .iter()
        .filter_map(|staged| match &staged.gain {
            Gain::Map(map) => Some((staged.list, map)),
            Gain::Radial(_) => None,
        })
        .collect()
}

fn radials(c: &DngCorrections) -> usize {
    c.gains
        .ops
        .iter()
        .filter(|staged| matches!(staged.gain, Gain::Radial(_)))
        .count()
}

#[test]
fn dji_fixture_carries_a_vignette_a_shared_warp_and_a_three_plane_map() {
    let c = corrections("DJI_FC7303_16-9.dng");
    assert_eq!(radials(&c), 1);
    let maps = maps(&c);
    assert_eq!(maps.len(), 1);
    assert_eq!(maps[0].0, OpcodeList::Three);
    let staged: Vec<bool> = c.gains.ops.iter().map(|g| g.after_warp).collect();
    assert_eq!(staged, vec![false, true]);
    assert_eq!((maps[0].1.map_planes, maps[0].1.points), (3, [49, 65]));
    let warp = c.warp.as_ref().expect("DJI warp");
    assert!(!warp.is_split());
    assert!((warp.planes[0].radial[0] - 0.9844).abs() < 1e-4);
    assert_eq!(warp.center, [0.5, 0.5]);
    assert!(matches!(c.embedded_lens(true), EmbeddedLens::Warp(_)));
}

#[test]
fn nexus_fixture_carries_four_list_two_phase_maps() {
    let c = corrections("LG_Nexus_5X_4-3.dng");
    let maps = maps(&c);
    assert_eq!(maps.len(), 4);
    let origins: Vec<[u32; 2]> = maps
        .iter()
        .map(|(_, m)| [m.area.top, m.area.left])
        .collect();
    assert_eq!(origins, vec![[0, 0], [0, 1], [1, 0], [1, 1]]);
    assert!(
        maps.iter()
            .all(|(list, m)| *list == OpcodeList::Two && m.pitch == [2, 2])
    );
    assert_eq!(radials(&c), 0);
    assert!(c.warp.is_none());
    assert_eq!(c.embedded_lens(true), EmbeddedLens::Shading);
}

#[test]
fn iphone_fixture_reads_its_vignette_from_ifd0() {
    let c = corrections("Apple_iPhone_8_4-3.dng");
    assert_eq!(radials(&c), 1);
    assert!(maps(&c).is_empty());
    let Some(Gain::Radial(radial)) = c.gains.ops.first().map(|staged| &staged.gain) else {
        panic!("iPhone vignette missing");
    };
    assert!((radial.center[0] - 0.4949).abs() < 1e-3);
    assert!((radial.k[0] - 2.2472).abs() < 1e-3);
}

#[test]
fn files_without_opcodes_carry_no_corrections() {
    for name in [
        "Canon_EOS_5D_3-2.cr2",
        "Ricoh_GR_12bit_4-3.dng",
        "Leica_M8_8bit_8bit_uncompressed_3-2.dng",
    ] {
        let c = corrections(name);
        assert!(!c.has_gains() && c.warp.is_none(), "{name}");
        assert_eq!(c.embedded_lens(true), EmbeddedLens::Absent, "{name}");
    }
    assert!(!DngCorrections::read(b"II*\0\x08\0\0\0").has_gains());
    assert!(!DngCorrections::read(b"not a tiff").has_gains());
}

#[test]
fn truncated_dng_never_panics() {
    let bytes = std::fs::read(fixture_path("Apple_iPhone_8_4-3.dng")).unwrap();
    for len in [8, 64, 512, 4096, bytes.len() / 2] {
        let _ = DngCorrections::read(&bytes[..len]);
    }
}

#[test]
fn opcodes_outside_their_stage_are_skipped() {
    let map = gain_map_payload([0, 0, 4, 4], [0, 1], [2, 2], &[2.0; 4]);
    let bad = words(&[0, 1, 0, 3, 5]);
    let warp = [
        words(&[1]),
        doubles(&[0.9, 0.1, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]),
    ]
    .concat();
    let c = DngCorrections::from_blobs([
        Blob::default().op(9, 0, &map).op(1, 0, &warp).bytes(),
        Blob::default().op(1, 0, &warp).bytes(),
        Blob::default()
            .op(5, 0, &bad)
            .op(4, 0, &words(&[0, 0]))
            .bytes(),
    ]);
    assert!(!c.has_gains());
    assert!(c.warp.is_none());
    assert!(c.bad_lists.is_empty() && c.bad_constant.is_none());
}

#[test]
fn only_the_first_warp_is_kept() {
    let warp = |k0: f64| {
        [
            words(&[1]),
            doubles(&[k0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.5]),
        ]
        .concat()
    };
    let c = DngCorrections::from_blobs([
        Vec::new(),
        Vec::new(),
        Blob::default()
            .op(1, 0, &warp(0.9))
            .op(1, 0, &warp(0.8))
            .bytes(),
    ]);
    let kept = c.warp.expect("warp");
    assert_eq!(kept.planes[2].radial[0], 0.9);
    assert_ne!(kept.planes[0], WarpPlane::IDENTITY);
}

fn develop(name: &str, corrected: bool, quality: bool) -> crate::frame::RawFrame {
    let bytes = std::fs::read(fixture_path(name)).unwrap();
    let source = rawler::rawsource::RawSource::new_from_slice(&bytes);
    let raw = rawler::decode(&source, &rawler::decoders::RawDecodeParams::default()).unwrap();
    let corrections = match corrected {
        true => DngCorrections::read(&bytes),
        false => DngCorrections::default(),
    };
    let develop = match quality {
        true => crate::decode::raw::decode_raw_quality,
        false => crate::decode::raw::decode_raw_fast,
    };
    develop(raw, None, &corrections).unwrap()
}

fn block_mean(frame: &crate::frame::RawFrame, [x0, y0]: [usize; 2]) -> f64 {
    let w = frame.meta.width;
    let cpp = frame.cpp;
    let sum: f64 = (y0..y0 + 64)
        .flat_map(|y| (x0 * cpp..(x0 + 64) * cpp).map(move |i| y * w * cpp + i))
        .map(|i| f64::from(frame.data[i]))
        .sum();
    sum / (64 * 64 * cpp) as f64
}

fn corner_lift(name: &str, quality: bool) -> f64 {
    let plain = develop(name, false, quality);
    let fixed = develop(name, true, quality);
    let w = fixed.meta.width;
    let h = fixed.meta.height;
    let corner = [16, 16];
    let centre = [(w / 2 - 32) & !1, (h / 2 - 32) & !1];
    let ratio = |f: &crate::frame::RawFrame| block_mean(f, corner) / block_mean(f, centre);
    ratio(&fixed) / ratio(&plain)
}

#[test]
fn embedded_gains_lift_phone_and_drone_corners_on_both_decode_paths() {
    for (name, expected) in [
        ("Apple_iPhone_8_4-3.dng", 2.6),
        ("LG_Nexus_5X_4-3.dng", 2.6),
        ("DJI_FC7303_16-9.dng", 1.8),
    ] {
        let fast = corner_lift(name, false);
        let quality = corner_lift(name, true);
        assert!(fast > expected, "{name}: fast corner lift {fast:.3}");
        assert!(
            (quality / fast - 1.0).abs() < 0.03,
            "{name}: quality {quality:.3} vs fast {fast:.3}"
        );
    }
}

#[test]
fn decoded_frames_report_their_embedded_lens() {
    let lens = |name: &str| {
        let bytes = std::fs::read(fixture_path(name)).unwrap();
        crate::decode::decode(&bytes).unwrap().embedded_lens
    };
    assert!(lens("DJI_FC7303_16-9.dng").warp().is_some());
    assert_eq!(lens("LG_Nexus_5X_4-3.dng"), EmbeddedLens::Shading);
    assert_eq!(lens("Apple_iPhone_8_4-3.dng"), EmbeddedLens::Shading);
    assert_eq!(lens("Canon_EOS_5D_3-2.cr2"), EmbeddedLens::Absent);
}

#[test]
fn hostile_bad_pixel_lists_stop_at_the_budget() {
    let everything = words(&[0, 0, 1, 0, 0, u32::MAX, u32::MAX]);
    let lists = (0..64).fold(Blob::default(), |blob, _| blob.op(5, 0, &everything));
    let c = DngCorrections::from_blobs([lists.bytes(), Vec::new(), Vec::new()]);
    let bytes = std::fs::read(fixture_path("Sony_ILCE-7S_14bit_14bit_compressed_3-2.arw")).unwrap();
    let source = rawler::rawsource::RawSource::new_from_slice(&bytes);
    let raw = rawler::decode(&source, &rawler::decoders::RawDecodeParams::default()).unwrap();
    assert_eq!(c.flagged_pixels(&raw).len(), 1 << 20);
}
