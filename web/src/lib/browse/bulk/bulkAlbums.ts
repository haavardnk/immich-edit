import { addAssetsToAlbum, removeAssetsFromAlbum } from '#lib/api/albums.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import { countOf } from '#lib/utils/plural.ts';

export async function addToAlbums(
  ids: string[],
  albumIds: readonly string[],
  noun = 'selected photo'
): Promise<boolean> {
  if (albumIds.length === 0 || !(await metadataConsent.gate())) return false;
  let failed = 0;
  for (const albumId of albumIds) {
    try {
      const results = await addAssetsToAlbum(albumId, ids);
      failed += results.filter((r) => !r.success && r.error !== 'duplicate').length;
    } catch {
      failed += ids.length;
    }
  }
  assetAlbums.forget(ids);
  if (failed > 0) {
    toasts.push('warn', `${failed} album additions failed`, { ttlMs: 6000 });
    return false;
  }
  const photos = countOf(ids.length, noun);
  const where = albumIds.length === 1 ? 'the album' : `${albumIds.length} albums`;
  toasts.push('success', `Added ${photos} to ${where}`, { ttlMs: 4000 });
  return true;
}

export async function removeFromAlbum(ids: string[], album: AlbumSummary): Promise<void> {
  if (!(await metadataConsent.gate())) return;
  try {
    const results = await removeAssetsFromAlbum(album.id, ids);
    assetAlbums.forget(ids);
    const removed = new Set(results.filter((r) => r.success).map((r) => r.id));
    browsing.assets
      .filter((a) => removed.has(sourceId(a.id)))
      .forEach((a) => browsing.remove(a.id));
    selection.clear();
    const failed = results.length - removed.size;
    if (failed > 0)
      toasts.push('warn', `${failed} could not be removed from ${album.albumName}`, {
        ttlMs: 6000
      });
    else toasts.push('success', `Removed ${removed.size} from ${album.albumName}`, { ttlMs: 4000 });
  } catch (e) {
    toasts.fail('Failed to remove from album', e, 6000);
  }
}
