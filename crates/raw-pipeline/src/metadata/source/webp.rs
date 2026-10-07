use little_exif::metadata::Metadata;

use super::exif_block;

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    chunks(data)
        .filter(|(kind, _)| *kind == b"EXIF")
        .find_map(|(_, body)| exif_block(body))
}

fn chunks(data: &[u8]) -> impl Iterator<Item = (&[u8], &[u8])> {
    let mut pos = 12usize;
    std::iter::from_fn(move || {
        let kind = data.get(pos..pos + 4)?;
        let len = u32::from_le_bytes(data.get(pos + 4..pos + 8)?.try_into().ok()?) as usize;
        let body = data.get(pos + 8..(pos + 8).checked_add(len)?)?;
        pos += 8 + len + (len & 1);
        Some((kind, body))
    })
}
