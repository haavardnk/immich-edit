mod boxes;
mod cr3;
mod heif;
mod jpeg;
mod jxl;
mod png;
mod raf;
mod tiff;
mod webp;

use std::io::Read;

use little_exif::metadata::Metadata;

use super::exif::parse_tiff;

const MAX_BLOCK_BYTES: u64 = 16 << 20;

pub fn read(data: &[u8]) -> Option<Metadata> {
    match data {
        [0xFF, 0xD8, ..] => jpeg::read(data),
        [0x89, b'P', b'N', b'G', ..] => png::read(data),
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => webp::read(data),
        [b'I', b'I', ..] | [b'M', b'M', ..] => tiff::read(data),
        [0, 0, 0, 0x0C, b'J', b'X', b'L', b' ', ..] => jxl::read(data),
        [
            _,
            _,
            _,
            _,
            b'f',
            b't',
            b'y',
            b'p',
            b'c',
            b'r',
            b'x',
            b' ',
            ..,
        ] => cr3::read(data),
        [_, _, _, _, b'f', b't', b'y', b'p', ..] => heif::read(data),
        _ if data.starts_with(raf::MAGIC) => raf::read(data),
        _ => None,
    }
}

fn exif_block(block: &[u8]) -> Option<Metadata> {
    parse_tiff(block.strip_prefix(b"Exif\0\0").unwrap_or(block))
}

fn exif_after_offset(block: &[u8]) -> Option<Metadata> {
    let offset = be_u32(block, 0)? as usize;
    exif_block(block.get(offset.checked_add(4)?..)?)
}

fn read_capped(reader: impl Read) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    reader
        .take(MAX_BLOCK_BYTES + 1)
        .read_to_end(&mut out)
        .ok()?;
    (out.len() as u64 <= MAX_BLOCK_BYTES).then_some(out)
}

fn be_u32(data: &[u8], pos: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        data.get(pos..pos.checked_add(4)?)?.try_into().ok()?,
    ))
}
