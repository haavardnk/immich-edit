import { addAssetsToAlbum, removeAssetsFromAlbum } from '#lib/api/albums.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import {
  NOT_IN_ALBUM,
  NOT_YOURS,
  removalText,
  type AlbumRemoval
} from '#lib/browse/bulk/albumRemoval.ts';
import { album } from '#lib/stores/album.svelte.ts';
import { assetAlbums } from '#lib/stores/assetAlbums.svelte.ts';
import { browsing, type Removal } from '#lib/stores/browsing.svelte.ts';
import { libraryChanged } from '#lib/stores/libraryChange.ts';
import { metadataConsent } from '#lib/stores/metadataConsent.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { dropFromView } from '#lib/stores/viewRemoval.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
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

function refreshAlbums(albumIds: Iterable<string>): void {
  for (const id of new Set(albumIds)) void album.refresh(id);
}

export async function addToAlbums(
  ids: string[],
  albumIds: readonly string[],
  noun = 'selected photo'
): Promise<boolean> {
  if (albumIds.length === 0 || !(await metadataConsent.gate())) return false;
  const failed = await addEach(ids, albumIds);
  assetAlbums.forget(ids);
  refreshAlbums(albumIds);
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

export async function removeFromAlbum(albumId: string, ids: string[]): Promise<AlbumRemoval> {
  const results = await removeAssetsFromAlbum(albumId, ids);
  const outcome: AlbumRemoval = { removed: [], refused: 0, failed: 0 };
  for (const result of results) {
    if (result.success) outcome.removed.push(result.id);
    else if (result.error === NOT_YOURS) outcome.refused += 1;
    else if (result.error !== NOT_IN_ALBUM) outcome.failed += 1;
  }
  return outcome;
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
      const outcome = await removeFromAlbum(albumId, ids);
      failed += outcome.refused + outcome.failed;
      if (albumId === viewing) for (const id of outcome.removed) gone.add(id);
    } catch {
      failed += ids.length;
    }
  }
  assetAlbums.forget(ids);
  refreshAlbums([...add, ...remove]);
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

async function undoRemoval(
  target: AlbumSummary,
  removed: string[],
  removal: Removal
): Promise<void> {
  if (await addToAlbums(removed, [target.id], 'photo')) browsing.reinsert(removal);
}

export async function removeFromViewedAlbum(target: AlbumSummary, ids: string[]): Promise<boolean> {
  if (ids.length === 0 || !(await metadataConsent.gate())) return false;
  let outcome: AlbumRemoval;
  try {
    outcome = await removeFromAlbum(target.id, ids);
  } catch (e) {
    toasts.fail('Remove from album', e);
    return false;
  }
  assetAlbums.forget(ids);
  if (outcome.removed.length > 0) void album.refresh(target.id);
  const gone = new Set(outcome.removed);
  const removal = dropFromView(
    new Set(browsing.assets.filter((a) => gone.has(sourceId(a.id))).map((a) => a.id))
  );
  selection.clear();
  const text = removalText(outcome, target.albumName);
  const undo =
    outcome.removed.length > 0
      ? { label: 'Undo', run: () => void undoRemoval(target, outcome.removed, removal) }
      : undefined;
  toasts.push(text.kind, text.message, { description: text.description, action: undo });
  return outcome.removed.length > 0;
}
