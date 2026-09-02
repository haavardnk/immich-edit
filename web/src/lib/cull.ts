import { updateAsset } from '$lib/api/assets';
import { addTagToAsset, removeTagFromAsset } from '$lib/api/tags';
import { browsing } from '$lib/stores/browsing.svelte';
import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
import { rejected } from '$lib/stores/rejected.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
import { ensureRejectTag, isRejected, setRejectedTags } from '$lib/reject';
import type { AssetSummary } from '$lib/types/album';
import type { ExifInfo } from '$lib/types/asset';

function withRating(asset: AssetSummary, rating: number | null): ExifInfo {
  if (asset.exifInfo) return { ...asset.exifInfo, rating };
  return {
    make: null,
    model: null,
    lensModel: null,
    fNumber: null,
    focalLength: null,
    iso: null,
    exposureTime: null,
    exifImageWidth: null,
    exifImageHeight: null,
    orientation: null,
    dateTimeOriginal: null,
    rating,
    fileSizeInByte: null
  };
}

export async function rateAsset(id: string, rating: number | null): Promise<boolean> {
  const asset = browsing.assets.find((a) => a.id === id);
  if (!asset) return false;
  if (!(await metadataConsent.gate())) return false;
  const prev = asset.exifInfo;
  browsing.patch(id, { exifInfo: withRating(asset, rating) });
  try {
    await updateAsset(id, { rating });
  } catch (e) {
    browsing.patch(id, { exifInfo: prev });
    toasts.push('error', `rating: ${(e as Error).message}`);
  }
  return true;
}

export async function toggleFavorite(id: string): Promise<boolean> {
  const asset = browsing.assets.find((a) => a.id === id);
  if (!asset) return false;
  if (!(await metadataConsent.gate())) return false;
  const next = !asset.isFavorite;
  browsing.patch(id, { isFavorite: next });
  try {
    await updateAsset(id, { isFavorite: next });
  } catch (e) {
    browsing.patch(id, { isFavorite: !next });
    toasts.push('error', `favorite: ${(e as Error).message}`);
  }
  return true;
}

export async function clearFlags(id: string): Promise<boolean> {
  const asset = browsing.assets.find((a) => a.id === id);
  if (!asset) return false;
  let changed = false;
  if (asset.isFavorite) changed = (await toggleFavorite(id)) || changed;
  if (isRejected(asset)) changed = (await toggleReject(id)) || changed;
  return changed;
}

export async function toggleReject(id: string): Promise<boolean> {
  const asset = browsing.assets.find((a) => a.id === id);
  if (!asset) return false;
  if (!(await metadataConsent.gate())) return false;
  const rejectTag = await ensureRejectTag();
  if (!rejectTag) {
    toasts.push('error', 'reject: could not create tag');
    return false;
  }
  const next = !isRejected(asset);
  const prev = asset.tags;
  browsing.patch(id, { tags: setRejectedTags(asset.tags, rejectTag, next) });
  if (next) rejected.add(id, rejectTag);
  else rejected.remove(id);
  try {
    if (next) await addTagToAsset(rejectTag.id, id);
    else await removeTagFromAsset(rejectTag.id, id);
  } catch (e) {
    browsing.patch(id, { tags: prev });
    if (next) rejected.remove(id);
    else rejected.add(id, rejectTag);
    toasts.push('error', `reject: ${(e as Error).message}`);
  }
  return true;
}
