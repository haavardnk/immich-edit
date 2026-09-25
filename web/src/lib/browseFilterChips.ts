import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';
import { LABEL_NAMES, type LabelColor } from '$lib/labels';

export type FilterKey = keyof BrowseFilters;

export interface FilterChip {
  key: FilterKey;
  label: string;
  color?: LabelColor;
}

const VISIBILITY_LABELS: Record<BrowseFilters['visibility'], string> = {
  timeline: 'Timeline',
  archive: 'Archived',
  hidden: 'Hidden'
};

function ratingLabel(rating: BrowseFilters['rating']): string {
  if (rating === 'unrated') return 'Unrated';
  return `${rating} star${rating === 1 ? '' : 's'}`;
}

export function activeFilterChips(filters: BrowseFilters): FilterChip[] {
  const chips: FilterChip[] = [];
  if (filters.visibility !== FILTER_DEFAULTS.visibility)
    chips.push({ key: 'visibility', label: VISIBILITY_LABELS[filters.visibility] });
  if (filters.rating !== FILTER_DEFAULTS.rating)
    chips.push({ key: 'rating', label: ratingLabel(filters.rating) });
  if (filters.favoriteOnly) chips.push({ key: 'favoriteOnly', label: 'Favorites' });
  if (filters.excludeRejected) chips.push({ key: 'excludeRejected', label: 'No rejected' });
  if (filters.label === 'none') chips.push({ key: 'label', label: 'No label' });
  else if (filters.label !== 'any')
    chips.push({
      key: 'label',
      label: `${LABEL_NAMES[filters.label]} label`,
      color: filters.label
    });
  if (filters.filename) chips.push({ key: 'filename', label: `Name: ${filters.filename}` });
  if (filters.takenAfter) chips.push({ key: 'takenAfter', label: `After ${filters.takenAfter}` });
  if (filters.takenBefore)
    chips.push({ key: 'takenBefore', label: `Before ${filters.takenBefore}` });
  return chips;
}

export function withoutFilter(filters: BrowseFilters, key: FilterKey): BrowseFilters {
  return { ...filters, [key]: FILTER_DEFAULTS[key] };
}

export function photoCountLabel(total: number, hidden: number): string {
  const photos = `${total} photo${total === 1 ? '' : 's'}`;
  return hidden > 0 ? `${photos}, ${hidden} hidden` : photos;
}
