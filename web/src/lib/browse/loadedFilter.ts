import { browseControls } from '$lib/stores/browseControls.svelte';
import type { AssetSummary } from '$lib/types/album';
import type { BrowseFilter } from '$lib/types/search';
import { sortAssets, timeOf, type SortKey } from './sortAssets';

const fold = (text: string): string => text.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();

function within(value: number | null | undefined, min?: number, max?: number): boolean {
  if (min === undefined && max === undefined) return true;
  if (value == null) return false;
  return (min === undefined || value >= min) && (max === undefined || value <= max);
}

export function keepsLoaded(filter: BrowseFilter, asset: AssetSummary): boolean {
  if (filter.favorite && !asset.isFavorite) return false;
  const name = filter.filename?.trim();
  if (name && !fold(asset.originalFileName).includes(fold(name))) return false;
  const after = filter.takenAfter ? Date.parse(filter.takenAfter) : undefined;
  const before = filter.takenBefore ? Date.parse(filter.takenBefore) : undefined;
  if (!within(timeOf(asset.fileCreatedAt), after, before)) return false;
  return within(asset.exifInfo?.fileSizeInByte, filter.minSize, filter.maxSize);
}

export function arrangeLoaded(assets: AssetSummary[], key: SortKey): AssetSummary[] {
  const filter = browseControls.filter({});
  const kept = assets.filter((asset) => keepsLoaded(filter, asset));
  return sortAssets(kept, key, browseControls.sortDir);
}
