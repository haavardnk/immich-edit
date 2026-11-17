import { addAssetsToAlbum, removeAssetsFromAlbum } from '#lib/api/albums.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { countOf } from '#lib/utils/plural.ts';

async function addEach(ids: string[], albumIds: readonly string[]): Promise<number> {
  let failed = 0;
  for (const albumId of albumIds) {
    try {
      const results = await addAssetsToAlbum(albumId, ids);
      failed += results.filter((r) => !r.success && r.error !== 'duplicate').length;
    } catch {
      failed += ids.length;
    }
  }
  return failed;
}

export async function addToAlbums(
  ids: string[],
  albumIds: readonly string[],
  noun = 'selected photo'
): Promise<boolean> {
  if (albumIds.length === 0 || !(await metadataConsent.gate())) return false;
  const failed = await addEach(ids, albumIds);
  assetAlbums.forget(ids);
  libraryChanged();
  if (failed > 0) {
    toasts.push('warn', `${failed} album additions failed`);
    return false;
  }
  const photos = countOf(ids.length, noun);
  const where = albumIds.length === 1 ? 'the album' : `${albumIds.length} albums`;
  toasts.push('success', `Added ${photos} to ${where}`);
  return true;
}

export async function changeAlbums(
  ids: string[],
  add: readonly string[],
  remove: readonly string[],
  viewing: string | null
): Promise<boolean> {
  if (add.length + remove.length === 0 || !(await metadataConsent.gate())) return false;
  let failed = await addEach(ids, add);
  const gone = new Set<string>();
  for (const albumId of remove) {
    try {
      const results = await removeAssetsFromAlbum(albumId, ids);
      failed += results.filter((r) => !r.success && r.error !== 'not_found').length;
      if (albumId === viewing) for (const r of results) if (r.success) gone.add(r.id);
    } catch {
      failed += ids.length;
    }
  }
  assetAlbums.forget(ids);
  libraryChanged();
  if (gone.size > 0) {
    browsing.assets.filter((a) => gone.has(sourceId(a.id))).forEach((a) => browsing.remove(a.id));
    selection.clear();
  }
  if (failed > 0) {
    toasts.push('warn', `${failed} album changes failed`);
    return false;
  }
  toasts.push('success', `Updated albums for ${countOf(ids.length, 'photo')}`);
  return true;
}
