use libheif_rs::{HeifContext, ImageHandle, ItemId};
use little_exif::metadata::Metadata;

use super::exif_after_offset;

pub(super) fn read(data: &[u8]) -> Option<Metadata> {
    let context = HeifContext::read_from_bytes(data).ok()?;
    let handle = context.primary_image_handle().ok()?;
    blocks(&handle, b"Exif").find_map(|block| exif_after_offset(&block))
}

fn blocks<'a>(
    handle: &'a ImageHandle,
    kind: &'static [u8; 4],
) -> impl Iterator<Item = Vec<u8>> + 'a {
    let count = usize::try_from(handle.number_of_metadata_blocks(kind)).unwrap_or(0);
    let mut ids: Vec<ItemId> = vec![0; count];
    let found = handle.metadata_block_ids(&mut ids, kind);
    ids.truncate(found);
    ids.into_iter()
        .filter_map(move |id| handle.metadata(id).ok())
}
