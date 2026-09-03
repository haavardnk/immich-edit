import { readStored, writeStored } from '$lib/utils/storage';
import { FILTER_DEFAULTS, type BrowseFilters, type RatingFilter } from './browseControls.svelte';
import type { Visibility } from '$lib/types/search';

const KEY = 'immich-edit:browseContext';

interface BrowseContextSnapshot {
  path: string;
  filters: BrowseFilters;
}

const RATINGS: RatingFilter[] = ['any', 'unrated', 1, 2, 3, 4, 5];
const VISIBILITIES: Visibility[] = ['timeline', 'archive', 'hidden'];

function sanitize(filters: Partial<BrowseFilters>): BrowseFilters {
  return {
    favoriteOnly: filters.favoriteOnly === true,
    rating: RATINGS.find((r) => r === filters.rating) ?? FILTER_DEFAULTS.rating,
    filename: typeof filters.filename === 'string' ? filters.filename : '',
    visibility: VISIBILITIES.find((v) => v === filters.visibility) ?? FILTER_DEFAULTS.visibility,
    takenAfter: typeof filters.takenAfter === 'string' ? filters.takenAfter : '',
    takenBefore: typeof filters.takenBefore === 'string' ? filters.takenBefore : '',
    excludeRejected: filters.excludeRejected === true
  };
}

export function rememberBrowseContext(path: string, filters: BrowseFilters): void {
  writeStored(KEY, { path, filters });
}

export function recallBrowseFilters(path: string): BrowseFilters | null {
  const stored = readStored<BrowseContextSnapshot>(KEY);
  if (!stored || stored.path !== path || !stored.filters) return null;
  return sanitize(stored.filters);
}
