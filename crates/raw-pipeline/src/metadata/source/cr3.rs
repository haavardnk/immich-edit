use little_exif::metadata::Metadata;

use super::boxes::boxes;
use super::parse_tiff;
use crate::metadata::exif::merge;

const EXIF_POINTER: u16 = 0x8769;
const GPS_POINTER: u16 = 0x8825;
const LONG: u16 = 4;

const CANON_UUID: [u8; 16] = [
    0x85, 0xc0, 0xb6, 0x87, 0x82, 0x0f, 0x11, 0xe0, 0x81, 0x11, 0xf4, 0xce, 0x46, 0x2b, 0x6a, 0x48,
];

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    boxes(data)
        .filter(|(kind, _)| *kind == b"moov")
        .find_map(|(_, body)| canon_exif(body))
}

fn canon_exif(moov: &[u8]) -> Option<Metadata> {
    let canon = boxes(moov).find_map(|(kind, body)| {
        (kind == b"uuid")
            .then(|| body.strip_prefix(&CANON_UUID))
            .flatten()
    })?;
    let mut out = None;
    for (kind, tiff) in boxes(canon) {
        let parsed = match kind {
            b"CMT1" => parse_tiff(tiff),
            b"CMT2" => as_sub_ifd(tiff, EXIF_POINTER).and_then(|t| parse_tiff(&t)),
            b"CMT4" => as_sub_ifd(tiff, GPS_POINTER).and_then(|t| parse_tiff(&t)),
            _ => continue,
        };
        out = merge(out, parsed);
    }
    out
}

fn as_sub_ifd(tiff: &[u8], pointer: u16) -> Option<Vec<u8>> {
    let little = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_bytes = |v: u16| {
        if little {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        }
    };
    let u32_bytes = |v: u32| {
        if little {
            v.to_le_bytes()
        } else {
            v.to_be_bytes()
        }
    };
    let first = tiff.get(4..8)?.to_vec();
    let mut out = tiff.to_vec();
    out.resize(out.len().next_multiple_of(2), 0);
    let root = u32::try_from(out.len()).ok()?;
    out.extend(u16_bytes(1));
    out.extend(u16_bytes(pointer));
    out.extend(u16_bytes(LONG));
    out.extend(u32_bytes(1));
    out.extend(first);
    out.extend(u32_bytes(0));
    out[4..8].copy_from_slice(&u32_bytes(root));
    Some(out)
}
