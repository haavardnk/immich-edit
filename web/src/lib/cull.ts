import { updateAsset } from '$lib/api/assets';
import { browsing } from '$lib/stores/browsing.svelte';
import { toasts } from '$lib/stores/toasts.svelte';
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
    dateTimeOriginal: null,
    rating,
    fileSizeInByte: null
  };
}

export async function rateAsset(id: string, rating: number | null): Promise<void> {
  const asset = browsing.assets.find((a) => a.id === id);
  if (!asset) return;
  const prev = asset.exifInfo;
  browsing.patch(id, { exifInfo: withRating(asset, rating) });
  try {
    await updateAsset(id, { rating });
  } catch (e) {
    browsing.patch(id, { exifInfo: prev });
    toasts.push('error', `rating: ${(e as Error).message}`);
  }
}

export async function toggleFavorite(id: string): Promise<void> {
  const asset = browsing.assets.find((a) => a.id === id);
  if (!asset) return;
  const next = !asset.isFavorite;
  browsing.patch(id, { isFavorite: next });
  try {
    await updateAsset(id, { isFavorite: next });
  } catch (e) {
    browsing.patch(id, { isFavorite: !next });
    toasts.push('error', `favorite: ${(e as Error).message}`);
  }
}
