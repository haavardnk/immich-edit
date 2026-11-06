import {
  FILTER_DEFAULTS,
  isDefaultFilter,
  type BrowseFilters,
  type FilterKey,
  type LabelFilter,
  type RejectedFilter
} from '#lib/stores/browseControls.svelte.ts';
import { LABEL_NAMES, type LabelColor } from '#lib/stores/labels.ts';
import { ratingOp, ratingStars } from '#lib/browse/rating.ts';
import { EXIF_LABELS, exifSummary, type ExifFilterKey } from '#lib/browse/exifFilters.ts';
import { ID_EXCLUSIONS, idChipLabel, type IdFilterKey } from '#lib/browse/idFilters.ts';
import { countOf } from '#lib/utils/plural.ts';

export interface FilterChip {
  key: FilterKey;
  label: string;
  color?: LabelColor;
}

export type FilterNameOf = (key: IdFilterKey, id: string) => string;

type ChipValue = Omit<FilterChip, 'key'>;
type ChipBuilder = (filters: BrowseFilters, nameOf: FilterNameOf) => ChipValue;

const VISIBILITY_LABELS: Record<BrowseFilters['visibility'], string> = {
  timeline: 'Timeline',
  archive: 'Archived',
  hidden: 'Hidden'
};

const REJECTED_LABELS: Record<RejectedFilter, string> = {
  any: 'Rejected shown',
  hide: 'No rejected',
  only: 'Rejected only'
};

function ratingLabel(rating: BrowseFilters['rating']): string {
  if (rating === 'any') return 'Any rating';
  if (rating === 'unrated') return 'Unrated';
  const stars = ratingStars(rating);
  const name = countOf(stars, 'star');
  const op = ratingOp(rating);
  if (op === 'gte') return `≥ ${name}`;
  if (op === 'lte') return `≤ ${name}`;
  return name;
}

function labelChip(label: LabelFilter): ChipValue {
  if (label === 'any') return { label: 'Any label' };
  if (label === 'none') return { label: 'No label' };
  return { label: `${LABEL_NAMES[label]} label`, color: label };
}

function idChip(key: IdFilterKey): ChipBuilder {
  return (filters, nameOf) => ({
    label: idChipLabel(key, filters[key], (id) => nameOf(key, id))
  });
}

function exifChip(key: ExifFilterKey): ChipBuilder {
  return (filters) => ({ label: `${EXIF_LABELS[key]}: ${exifSummary(filters[key])}` });
}

const CHIPS: Record<FilterKey, ChipBuilder> = {
  visibility: (filters) => ({ label: VISIBILITY_LABELS[filters.visibility] }),
  rating: (filters) => ({ label: ratingLabel(filters.rating) }),
  favoriteOnly: () => ({ label: 'Favorites' }),
  notInAlbum: () => ({ label: ID_EXCLUSIONS.albums.label }),
  untagged: () => ({ label: ID_EXCLUSIONS.tags.label }),
  noPeople: () => ({ label: ID_EXCLUSIONS.people.label }),
  albums: idChip('albums'),
  people: idChip('people'),
  tags: idChip('tags'),
  rejected: (filters) => ({ label: REJECTED_LABELS[filters.rejected] }),
  label: (filters) => labelChip(filters.label),
  filename: (filters) => ({ label: `Name: ${filters.filename}` }),
  minSize: (filters) => ({ label: `≥ ${filters.minSize} MB` }),
  maxSize: (filters) => ({ label: `≤ ${filters.maxSize} MB` }),
  takenAfter: (filters) => ({ label: `After ${filters.takenAfter}` }),
  takenBefore: (filters) => ({ label: `Before ${filters.takenBefore}` }),
  addedAfter: (filters) => ({ label: `Added after ${filters.addedAfter}` }),
  addedBefore: (filters) => ({ label: `Added before ${filters.addedBefore}` }),
  make: exifChip('make'),
  model: exifChip('model'),
  lensModel: exifChip('lensModel'),
  country: exifChip('country'),
  state: exifChip('state'),
  city: exifChip('city')
};

const CHIP_ORDER = Object.keys(CHIPS) as FilterKey[];

export function filterValueLabel(
  filters: BrowseFilters,
  key: FilterKey,
  nameOf: FilterNameOf
): string {
  return CHIPS[key](filters, nameOf).label;
}

export function filterChip(
  filters: BrowseFilters,
  key: FilterKey,
  nameOf: FilterNameOf
): FilterChip | undefined {
  if (isDefaultFilter(filters, key)) return undefined;
  return { key, ...CHIPS[key](filters, nameOf) };
}

export function activeFilterChips(filters: BrowseFilters, nameOf: FilterNameOf): FilterChip[] {
  return CHIP_ORDER.flatMap((key) => filterChip(filters, key, nameOf) ?? []);
}

export function withoutFilter(filters: BrowseFilters, key: FilterKey): BrowseFilters {
  return { ...filters, [key]: FILTER_DEFAULTS[key] };
}

export function photoCountLabel(total: number, hidden: number): string {
  const photos = countOf(total, 'photo');
  return hidden > 0 ? `${photos}, ${hidden} hidden` : photos;
}

export function showPhotosLabel(total: number | undefined): string {
  return total === undefined ? 'Show photos' : `Show ${countOf(total, 'photo')}`;
}
