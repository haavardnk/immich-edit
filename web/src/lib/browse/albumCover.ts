import { sourceId } from '#lib/browse/assetKey.ts';
import { editableViewedAlbum } from '#lib/browse/viewedAlbum.ts';
import { album } from '#lib/stores/album.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';

export function canSetViewedAlbumCover(id: string | null): boolean {
  const target = editableViewedAlbum();
  return target !== null && id !== null && target.albumThumbnailAssetId !== sourceId(id);
}

export async function setViewedAlbumCover(id: string): Promise<boolean> {
  const target = editableViewedAlbum();
  if (!target) return false;
  const updated = await album.update(target.id, { albumThumbnailAssetId: id }, 'Set album cover');
  if (!updated) return false;
  toasts.push('success', `Set as cover of ${updated.albumName}`);
  return true;
}
