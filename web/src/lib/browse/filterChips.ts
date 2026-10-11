import { FILTER_DEFAULTS, type BrowseFilters } from '$lib/stores/browseControls.svelte';
import { LABEL_NAMES, type LabelColor } from '$lib/stores/labels';
import { ratingOp, ratingStars } from '$lib/browse/rating';
import { EXIF_FILTERS, EXIF_LABELS, exifSummary } from '$lib/browse/exifFilters';

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
  const stars = ratingStars(rating);
  const name = `${stars} star${stars === 1 ? '' : 's'}`;
  const op = ratingOp(rating);
  if (op === 'gte') return `≥ ${name}`;
  if (op === 'lte') return `≤ ${name}`;
  return name;
}

export function activeFilterChips(filters: BrowseFilters): FilterChip[] {
  const chips: FilterChip[] = [];
  if (filters.visibility !== FILTER_DEFAULTS.visibility)
    chips.push({ key: 'visibility', label: VISIBILITY_LABELS[filters.visibility] });
  if (filters.rating !== FILTER_DEFAULTS.rating)
    chips.push({ key: 'rating', label: ratingLabel(filters.rating) });
  if (filters.favoriteOnly) chips.push({ key: 'favoriteOnly', label: 'Favorites' });
  if (filters.rejected !== 'any')
    chips.push({
      key: 'rejected',
      label: filters.rejected === 'hide' ? 'No rejected' : 'Rejected only'
    });
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
  for (const key of EXIF_FILTERS) {
    if (filters[key].length > 0)
      chips.push({ key, label: `${EXIF_LABELS[key]}: ${exifSummary(filters[key])}` });
  }
  return chips;
}

export function withoutFilter(filters: BrowseFilters, key: FilterKey): BrowseFilters {
  return { ...filters, [key]: FILTER_DEFAULTS[key] };
}

export function photoCountLabel(total: number, hidden: number): string {
  const photos = `${total} photo${total === 1 ? '' : 's'}`;
  return hidden > 0 ? `${photos}, ${hidden} hidden` : photos;
}
