import type { BrowseFilters } from '$lib/stores/browseControls.svelte';
import type { BrowseFilter } from '$lib/types/search';
import { isRange, ratingMatch } from './rating';
import { EXIF_FILTERS, UNKNOWN_EXIF } from './exifFilters';
import { toBytes } from './fileSize';

export function serverFilter(
  filters: BrowseFilters,
  base: BrowseFilter,
  structured: boolean
): BrowseFilter {
  const filter: BrowseFilter = { ...base, visibility: filters.visibility, imagesOnly: true };
  const rating = ratingMatch(isRange(filters.rating) && !structured ? 'any' : filters.rating);
  if (filters.favoriteOnly) filter.favorite = true;
  if (filters.notInAlbum) filter.notInAlbum = true;
  if (filters.untagged) filter.untagged = true;
  if (filters.noPeople) filter.noPeople = true;
  if (rating) filter.rating = rating;
  if (filters.filename) filter.filename = filters.filename;
  if (filters.minSize !== null) filter.minSize = toBytes(filters.minSize);
  if (filters.maxSize !== null) filter.maxSize = toBytes(filters.maxSize);
  if (filters.takenAfter) filter.takenAfter = new Date(filters.takenAfter).toISOString();
  if (filters.takenBefore) {
    filter.takenBefore = new Date(filters.takenBefore + 'T23:59:59').toISOString();
  }
  if (filters.rejected === 'only' || (filters.rejected === 'hide' && structured))
    filter.rejected = filters.rejected;
  if (filters.label !== 'any' && (filters.label !== 'none' || structured))
    filter.label = filters.label;
  for (const key of EXIF_FILTERS) {
    const values = filters[key];
    if (values.length === 1 || (values.length > 1 && structured))
      filter[key] = values.map((value) => (value === UNKNOWN_EXIF ? null : value));
  }
  return filter;
}
