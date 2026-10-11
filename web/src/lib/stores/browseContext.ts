import { readStored, writeStored } from '$lib/utils/storage';
import {
  FILTER_DEFAULTS,
  REJECTED_FILTERS,
  type BrowseFilters,
  type LabelFilter
} from './browseControls.svelte';
import { EXACT_RATINGS, RANGE_RATINGS, type RatingFilter } from '$lib/browse/rating';
import { immichCapabilities } from './immichCapabilities.svelte';
import type { Visibility } from '$lib/types/search';
import { LABEL_COLORS } from '$lib/stores/labels';

const KEY = 'immich-edit:browseContext';

interface BrowseContextSnapshot {
  path: string;
  filters: BrowseFilters;
}

const VISIBILITIES: Visibility[] = ['timeline', 'archive', 'hidden'];
const LABELS: LabelFilter[] = ['any', 'none', ...LABEL_COLORS];

function storedValues(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  const values = value.filter((v): v is string => typeof v === 'string' && v !== '');
  return values.length <= 1 || immichCapabilities.structuredSearch ? values : [];
}

function sanitize(filters: Partial<BrowseFilters> & { excludeRejected?: unknown }): BrowseFilters {
  const ratings: RatingFilter[] = immichCapabilities.structuredSearch
    ? [...EXACT_RATINGS, ...RANGE_RATINGS]
    : EXACT_RATINGS;
  return {
    favoriteOnly: filters.favoriteOnly === true,
    rating: ratings.find((r) => r === filters.rating) ?? FILTER_DEFAULTS.rating,
    filename: typeof filters.filename === 'string' ? filters.filename : '',
    visibility: VISIBILITIES.find((v) => v === filters.visibility) ?? FILTER_DEFAULTS.visibility,
    takenAfter: typeof filters.takenAfter === 'string' ? filters.takenAfter : '',
    takenBefore: typeof filters.takenBefore === 'string' ? filters.takenBefore : '',
    rejected:
      REJECTED_FILTERS.find((r) => r === filters.rejected) ??
      (filters.excludeRejected === true ? 'hide' : FILTER_DEFAULTS.rejected),
    label: LABELS.find((l) => l === filters.label) ?? FILTER_DEFAULTS.label,
    make: storedValues(filters.make),
    model: storedValues(filters.model),
    lensModel: storedValues(filters.lensModel),
    country: storedValues(filters.country),
    state: storedValues(filters.state),
    city: storedValues(filters.city)
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
