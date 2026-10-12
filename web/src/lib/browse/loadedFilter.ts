import { browseControls, type BrowseFilters } from '$lib/stores/browseControls.svelte';
import { labelOf } from '$lib/stores/labels';
import type { AssetSummary } from '$lib/types/album';
import type { BrowseFilter } from '$lib/types/search';
import { EXIF_FILTERS } from './exifFilters';
import { ratingHides } from './rating';
import { isRejected } from './reject';
import { sortAssets, timeOf, type SortKey } from './sortAssets';

export const LOADED_FILTERS: (keyof BrowseFilters)[] = [
  'favoriteOnly',
  'rating',
  'filename',
  'minSize',
  'maxSize',
  'takenAfter',
  'takenBefore',
  'addedAfter',
  'addedBefore',
  'rejected',
  'label',
  'make',
  'model',
  'lensModel',
  'country',
  'state',
  'city'
];

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
  const addedAfter = filter.addedAfter ? Date.parse(filter.addedAfter) : undefined;
  const addedBefore = filter.addedBefore ? Date.parse(filter.addedBefore) : undefined;
  if (!within(timeOf(asset.createdAt), addedAfter, addedBefore)) return false;
  return within(asset.exifInfo?.fileSizeInByte, filter.minSize, filter.maxSize);
}

export function hides(filters: BrowseFilters, asset: AssetSummary): boolean {
  if (ratingHides(filters.rating, asset.exifInfo?.rating)) return true;
  if (filters.rejected !== 'any' && isRejected(asset) !== (filters.rejected === 'only'))
    return true;
  const exif = asset.exifInfo;
  if (
    EXIF_FILTERS.some((key) => filters[key].length > 0 && !filters[key].includes(exif?.[key] ?? ''))
  )
    return true;
  if (filters.label === 'any') return false;
  return labelOf(asset) !== (filters.label === 'none' ? null : filters.label);
}

export function arrangeLoaded(assets: AssetSummary[], key: SortKey): AssetSummary[] {
  const filter = browseControls.filter({});
  const kept = assets.filter((asset) => keepsLoaded(filter, asset));
  return sortAssets(kept, key, browseControls.sortDir);
}
