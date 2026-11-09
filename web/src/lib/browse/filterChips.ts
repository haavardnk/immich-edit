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
import { ID_EXCLUSIONS, idChipLabel, idSummary, type IdFilterKey } from '#lib/browse/idFilters.ts';
import { dateChipLabel, dateRange, type DateBasis } from '#lib/browse/datePeriods.ts';
import { sizeLabel } from '#lib/browse/fileSize.ts';
import { countOf } from '#lib/utils/plural.ts';

export interface FilterChip {
  key: FilterKey;
  label: string;
  color?: LabelColor;
}

export type FilterNameOf = (key: IdFilterKey, id: string) => string;

type ChipValue = Omit<FilterChip, 'key'> & { value?: string };
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
  return (filters, nameOf) => {
    const name = (id: string): string => nameOf(key, id);
    const { ids, match } = filters[key];
    return {
      label: idChipLabel(key, filters[key], name),
      ...(match === 'all' ? { value: idSummary(ids, name) } : {})
    };
  };
}

function exifChip(key: ExifFilterKey): ChipBuilder {
  return (filters) => {
    const value = exifSummary(filters[key]);
    return { label: `${EXIF_LABELS[key]}: ${value}`, value };
  };
}

function dateChip(basis: DateBasis): ChipBuilder {
  return (filters) => ({ label: dateChipLabel(dateRange(filters, basis), basis) });
}

const sizeChip: ChipBuilder = (filters) => ({ label: sizeLabel(filters.minSize, filters.maxSize) });

const PAIRED: Partial<Record<FilterKey, FilterKey>> = {
  minSize: 'maxSize',
  maxSize: 'minSize',
  takenAfter: 'takenBefore',
  takenBefore: 'takenAfter',
  addedAfter: 'addedBefore',
  addedBefore: 'addedAfter'
};

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
  minSize: sizeChip,
  maxSize: sizeChip,
  takenAfter: dateChip('taken'),
  takenBefore: dateChip('taken'),
  addedAfter: dateChip('added'),
  addedBefore: dateChip('added'),
  make: exifChip('make'),
  model: exifChip('model'),
  lensModel: exifChip('lensModel'),
  country: exifChip('country'),
  state: exifChip('state'),
  city: exifChip('city')
};

const CHIP_ORDER = Object.keys(CHIPS) as FilterKey[];

function shownByPartner(filters: BrowseFilters, key: FilterKey): boolean {
  const partner = PAIRED[key];
  if (!partner || CHIP_ORDER.indexOf(partner) > CHIP_ORDER.indexOf(key)) return false;
  return !isDefaultFilter(filters, partner);
}

function chipShown(filters: BrowseFilters, key: FilterKey): boolean {
  return !isDefaultFilter(filters, key) && !shownByPartner(filters, key);
}

export function activeFilterKeys(filters: BrowseFilters): FilterKey[] {
  return CHIP_ORDER.filter((key) => chipShown(filters, key));
}

export function activeFilterCount(filters: BrowseFilters): number {
  return activeFilterKeys(filters).length;
}

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
  if (!chipShown(filters, key)) return undefined;
  const { label, color } = CHIPS[key](filters, nameOf);
  return color ? { key, label, color } : { key, label };
}

export function activeFilterChips(filters: BrowseFilters, nameOf: FilterNameOf): FilterChip[] {
  return activeFilterKeys(filters).flatMap((key) => filterChip(filters, key, nameOf) ?? []);
}

export function filterValue(
  filters: BrowseFilters,
  key: FilterKey,
  nameOf: FilterNameOf
): string | undefined {
  if (!chipShown(filters, key)) return undefined;
  const { label, value } = CHIPS[key](filters, nameOf);
  return value ?? label;
}

export function activeFilterValues(filters: BrowseFilters, nameOf: FilterNameOf): string[] {
  return activeFilterKeys(filters).flatMap((key) => filterValue(filters, key, nameOf) ?? []);
}

export function withoutFilter(filters: BrowseFilters, key: FilterKey): BrowseFilters {
  const partner = PAIRED[key];
  const cleared = { ...filters, [key]: FILTER_DEFAULTS[key] };
  return partner ? { ...cleared, [partner]: FILTER_DEFAULTS[partner] } : cleared;
}

export function photoCountLabel(total: number, hidden: number): string {
  const photos = countOf(total, 'photo');
  return hidden > 0 ? `${photos}, ${hidden} hidden` : photos;
}

export function showPhotosLabel(total: number | undefined): string {
  return total === undefined ? 'Show photos' : `Show ${countOf(total, 'photo')}`;
}
