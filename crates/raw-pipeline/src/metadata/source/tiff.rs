use std::borrow::Cow;

use little_exif::metadata::Metadata;

use super::{jpeg, parse_tiff};

const PANASONIC_JPEG_TAG: u16 = 0x002E;
const TIFF_MAGIC: u16 = 42;
const BIGTIFF_MAGIC: u16 = 43;

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    if data.get(2..4) == Some(b"U\0".as_slice()) {
        return ifd0_bytes(data, PANASONIC_JPEG_TAG).and_then(jpeg::read);
    }
    standard_magic(data).and_then(|tiff| parse_tiff(&tiff))
}

fn standard_magic(data: &[u8]) -> Option<Cow<'_, [u8]>> {
    let little = data.get(..2)? == b"II";
    let magic = u16_at(data, 2, little)?;
    if magic == TIFF_MAGIC {
        return Some(Cow::Borrowed(data));
    }
    if magic == BIGTIFF_MAGIC {
        return None;
    }
    let mut patched = data.to_vec();
    let bytes = if little {
        TIFF_MAGIC.to_le_bytes()
    } else {
        TIFF_MAGIC.to_be_bytes()
    };
    patched[2..4].copy_from_slice(&bytes);
    Some(Cow::Owned(patched))
}

fn ifd0_bytes(data: &[u8], wanted: u16) -> Option<&[u8]> {
    let little = data.get(..2)? == b"II";
    let ifd = u32_at(data, 4, little)? as usize;
    let count = u16_at(data, ifd, little)? as usize;
    let entry = (0..count)
        .map(|i| ifd + 2 + i * 12)
        .find(|&entry| u16_at(data, entry, little) == Some(wanted))?;
    if !matches!(u16_at(data, entry + 2, little)?, 1 | 7) {
        return None;
    }
    let len = u32_at(data, entry + 4, little)? as usize;
    let start = if len <= 4 {
        entry + 8
    } else {
        u32_at(data, entry + 8, little)? as usize
    };
    data.get(start..start.checked_add(len)?)
}

fn u16_at(data: &[u8], pos: usize, little: bool) -> Option<u16> {
    let bytes: [u8; 2] = data.get(pos..pos.checked_add(2)?)?.try_into().ok()?;
    Some(if little {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    })
}

fn u32_at(data: &[u8], pos: usize, little: bool) -> Option<u32> {
    let bytes: [u8; 4] = data.get(pos..pos.checked_add(4)?)?.try_into().ok()?;
    Some(if little {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    })
}
