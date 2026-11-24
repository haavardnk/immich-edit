import type { BrowseFilters } from '#lib/stores/browseControls.svelte.ts';
import type { BrowseFilter } from '#lib/types/search.ts';
import { isRange, ratingMatch } from './rating';
import { EXIF_FILTERS, UNKNOWN_EXIF } from './exifFilters';
import { toBytes } from './fileSize';
import { ID_FILTERS, idSet } from './idFilters';
import { endInstant, localSpan, startInstant } from './dateBounds';

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
  for (const key of ID_FILTERS) {
    const set = idSet(filters[key], structured);
    if (set) filter[key] = set;
  }
  if (rating) filter.rating = rating;
  if (filters.filename) filter.filename = filters.filename;
  if (filters.minSize !== null) filter.minSize = toBytes(filters.minSize);
  if (filters.maxSize !== null) filter.maxSize = toBytes(filters.maxSize);
  const taken = localSpan(filters.takenAfter, filters.takenBefore);
  if (taken) filter.takenLocal = taken;
  if (filters.addedAfter) filter.addedAfter = startInstant(filters.addedAfter);
  if (filters.addedBefore) filter.addedBefore = endInstant(filters.addedBefore);
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
