import { readStored, writeStored } from '$lib/utils/storage';
import {
  FILTER_DEFAULTS,
  RATING_VALUES,
  isMinRating,
  type BrowseFilters,
  type LabelFilter,
  type RejectedFilter
} from './browseControls.svelte';
import type { Visibility } from '$lib/types/search';
import { LABEL_COLORS } from '$lib/labels';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';

const KEY = 'immich-edit:browseContext';

interface BrowseContextSnapshot {
  path: string;
  filters: BrowseFilters;
}

const REJECTED: RejectedFilter[] = ['any', 'hide', 'only'];
const VISIBILITIES: Visibility[] = ['timeline', 'archive', 'hidden'];
const LABELS: LabelFilter[] = ['any', 'none', ...LABEL_COLORS];

function sanitize(filters: Partial<BrowseFilters> & { excludeRejected?: unknown }): BrowseFilters {
  return {
    favoriteOnly: filters.favoriteOnly === true,
    rating:
      RATING_VALUES.find(
        (r) => r === filters.rating && (immichCapabilities.minRatingFilter || !isMinRating(r))
      ) ?? FILTER_DEFAULTS.rating,
    filename: typeof filters.filename === 'string' ? filters.filename : '',
    visibility: VISIBILITIES.find((v) => v === filters.visibility) ?? FILTER_DEFAULTS.visibility,
    takenAfter: typeof filters.takenAfter === 'string' ? filters.takenAfter : '',
    takenBefore: typeof filters.takenBefore === 'string' ? filters.takenBefore : '',
    rejected:
      REJECTED.find((r) => r === filters.rejected) ??
      (filters.excludeRejected === true ? 'hide' : FILTER_DEFAULTS.rejected),
    label: LABELS.find((l) => l === filters.label) ?? FILTER_DEFAULTS.label
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
