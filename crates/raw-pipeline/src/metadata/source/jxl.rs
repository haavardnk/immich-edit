use std::borrow::Cow;

use super::boxes::boxes;
use little_exif::metadata::Metadata;

use super::{exif_after_offset, read_capped};

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    boxes(data)
        .filter_map(|(kind, body)| unwrap_brob(kind, body))
        .filter(|(kind, _)| *kind == b"Exif")
        .find_map(|(_, body)| exif_after_offset(&body))
}

fn unwrap_brob<'a>(kind: &'a [u8], body: &'a [u8]) -> Option<(&'a [u8], Cow<'a, [u8]>)> {
    if kind != b"brob" {
        return Some((kind, Cow::Borrowed(body)));
    }
    let inner = body.get(..4)?;
    if inner != b"Exif" {
        return None;
    }
    let stream = body.get(4..)?;
    let decompressed = read_capped(brotli_decompressor::Decompressor::new(stream, 4096))?;
    Some((inner, Cow::Owned(decompressed)))
}
