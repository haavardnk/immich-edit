use little_exif::metadata::Metadata;

use super::{be_u32, jpeg};

pub(super) const MAGIC: &[u8] = b"FUJIFILMCCD-RAW ";
const JPEG_OFFSET: usize = 84;

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    let embedded = be_u32(data, JPEG_OFFSET).zip(be_u32(data, JPEG_OFFSET + 4));
    embedded
        .and_then(|(offset, len)| {
            let start = offset as usize;
            data.get(start..start.checked_add(len as usize)?)
        })
        .and_then(jpeg::read)
}
