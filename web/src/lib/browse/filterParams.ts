import { DateTime } from 'luxon';
import {
  FILTER_DEFAULTS,
  isDefaultFilter,
  REJECTED_FILTERS,
  type BrowseFilters,
  type LabelFilter
} from '$lib/stores/browseControls.svelte';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { LABEL_COLORS } from '$lib/stores/labels';
import type { Visibility } from '$lib/types/search';
import { EXACT_RATINGS, RANGE_RATINGS, type RatingFilter } from './rating';
import { UNKNOWN_EXIF, type ExifFilterKey } from './exifFilters';
import { megabytes } from './fileSize';
import { structuredRoute, supportsFilter } from './browseRoutes';

const VISIBILITIES: Visibility[] = ['timeline', 'archive', 'hidden'];
const LABELS: LabelFilter[] = ['any', 'none', ...LABEL_COLORS];

const PARAMS: Record<keyof BrowseFilters, string> = {
  favoriteOnly: 'favorite',
  notInAlbum: 'noalbum',
  untagged: 'untagged',
  noPeople: 'nopeople',
  rating: 'rating',
  filename: 'filename',
  minSize: 'minsize',
  maxSize: 'maxsize',
  visibility: 'visibility',
  takenAfter: 'after',
  takenBefore: 'before',
  rejected: 'rejected',
  label: 'label',
  make: 'make',
  model: 'model',
  lensModel: 'lens',
  country: 'country',
  state: 'state',
  city: 'city'
};

const TYPED_KEYS: (keyof BrowseFilters)[] = [
  'filename',
  'minSize',
  'maxSize',
  'takenAfter',
  'takenBefore'
];

function day(value: string | null): string {
  if (!value || !/^\d{4}-\d{2}-\d{2}$/.test(value)) return '';
  return DateTime.fromISO(value).isValid ? value : '';
}

function values(params: URLSearchParams, key: ExifFilterKey): string[] {
  const all = [...new Set(params.getAll(PARAMS[key]))];
  const known = all.filter((value) => value !== UNKNOWN_EXIF);
  const read = known.length > 0 ? known : all;
  return read.length <= 1 || immichCapabilities.structuredSearch ? read : [];
}

export function readFilterParams(url: URL): BrowseFilters {
  const params = url.searchParams;
  const get = (key: keyof BrowseFilters): string | null => params.get(PARAMS[key]);
  const ratings: RatingFilter[] = structuredRoute(url)
    ? [...EXACT_RATINGS, ...RANGE_RATINGS]
    : EXACT_RATINGS;
  const filters: BrowseFilters = {
    favoriteOnly: get('favoriteOnly') === '1',
    notInAlbum: get('notInAlbum') === '1',
    untagged: get('untagged') === '1',
    noPeople: get('noPeople') === '1',
    rating: ratings.find((r) => String(r) === get('rating')) ?? FILTER_DEFAULTS.rating,
    filename: get('filename') ?? '',
    minSize: megabytes(get('minSize')),
    maxSize: megabytes(get('maxSize')),
    visibility: VISIBILITIES.find((v) => v === get('visibility')) ?? FILTER_DEFAULTS.visibility,
    takenAfter: day(get('takenAfter')),
    takenBefore: day(get('takenBefore')),
    rejected: REJECTED_FILTERS.find((r) => r === get('rejected')) ?? FILTER_DEFAULTS.rejected,
    label: LABELS.find((l) => l === get('label')) ?? FILTER_DEFAULTS.label,
    make: values(params, 'make'),
    model: values(params, 'model'),
    lensModel: values(params, 'lensModel'),
    country: values(params, 'country'),
    state: values(params, 'state'),
    city: values(params, 'city')
  };
  const unsupported = (Object.keys(filters) as (keyof BrowseFilters)[])
    .filter((key) => !supportsFilter(url, key))
    .map((key) => [key, FILTER_DEFAULTS[key]]);
  return { ...filters, ...Object.fromEntries(unsupported) };
}

export function withFilterParams(url: URL, filters: BrowseFilters): URL {
  const next = new URL(url);
  for (const name of Object.values(PARAMS)) next.searchParams.delete(name);
  for (const key of Object.keys(PARAMS) as (keyof BrowseFilters)[]) {
    if (isDefaultFilter(filters, key)) continue;
    const value = filters[key];
    const written = Array.isArray(value) ? value : [value === true ? '1' : String(value)];
    for (const item of written) next.searchParams.append(PARAMS[key], item);
  }
  return next;
}

export function isTypedEdit(from: BrowseFilters, to: BrowseFilters): boolean {
  const changed = (Object.keys(to) as (keyof BrowseFilters)[]).filter(
    (key) => JSON.stringify(from[key]) !== JSON.stringify(to[key])
  );
  return (
    changed.length > 0 && changed.every((key) => TYPED_KEYS.includes(key) && from[key] && to[key])
  );
}
