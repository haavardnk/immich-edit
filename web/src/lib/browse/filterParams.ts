import { DateTime } from 'luxon';
import {
  FILTER_DEFAULTS,
  REJECTED_FILTERS,
  type BrowseFilters,
  type LabelFilter
} from '$lib/stores/browseControls.svelte';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { LABEL_COLORS } from '$lib/stores/labels';
import type { Visibility } from '$lib/types/search';
import { EXACT_RATINGS, RANGE_RATINGS, type RatingFilter } from './rating';
import { EXIF_FILTERS, UNKNOWN_EXIF, type ExifFilterKey } from './exifFilters';
import { megabytes } from './fileSize';
import { supportsFilter } from './browseRoutes';

type ScalarKey = Exclude<keyof BrowseFilters, ExifFilterKey>;

const VISIBILITIES: Visibility[] = ['timeline', 'archive', 'hidden'];
const LABELS: LabelFilter[] = ['any', 'none', ...LABEL_COLORS];

const SCALAR_PARAMS: [string, ScalarKey][] = [
  ['favorite', 'favoriteOnly'],
  ['noalbum', 'notInAlbum'],
  ['untagged', 'untagged'],
  ['nopeople', 'noPeople'],
  ['rating', 'rating'],
  ['filename', 'filename'],
  ['minsize', 'minSize'],
  ['maxsize', 'maxSize'],
  ['visibility', 'visibility'],
  ['after', 'takenAfter'],
  ['before', 'takenBefore'],
  ['rejected', 'rejected'],
  ['label', 'label']
];

const EXIF_PARAMS: Record<ExifFilterKey, string> = {
  make: 'make',
  model: 'model',
  lensModel: 'lens',
  country: 'country',
  state: 'state',
  city: 'city'
};

const FILTER_PARAMS = [
  ...SCALAR_PARAMS.map(([name]) => name),
  ...EXIF_FILTERS.map((key) => EXIF_PARAMS[key])
];

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

function values(params: URLSearchParams, name: string): string[] {
  const all = [...new Set(params.getAll(name))];
  const known = all.filter((value) => value !== UNKNOWN_EXIF);
  const read = known.length > 0 ? known : all;
  return read.length <= 1 || immichCapabilities.structuredSearch ? read : [];
}

export function readFilterParams(url: URL): BrowseFilters {
  const params = url.searchParams;
  const ratings: RatingFilter[] = immichCapabilities.structuredSearch
    ? [...EXACT_RATINGS, ...RANGE_RATINGS]
    : EXACT_RATINGS;
  const filters: BrowseFilters = {
    favoriteOnly: params.get('favorite') === '1',
    notInAlbum: params.get('noalbum') === '1',
    untagged: params.get('untagged') === '1',
    noPeople: params.get('nopeople') === '1',
    rating: ratings.find((r) => String(r) === params.get('rating')) ?? FILTER_DEFAULTS.rating,
    filename: params.get('filename') ?? '',
    minSize: megabytes(params.get('minsize')),
    maxSize: megabytes(params.get('maxsize')),
    visibility:
      VISIBILITIES.find((v) => v === params.get('visibility')) ?? FILTER_DEFAULTS.visibility,
    takenAfter: day(params.get('after')),
    takenBefore: day(params.get('before')),
    rejected:
      REJECTED_FILTERS.find((r) => r === params.get('rejected')) ?? FILTER_DEFAULTS.rejected,
    label: LABELS.find((l) => l === params.get('label')) ?? FILTER_DEFAULTS.label,
    make: values(params, EXIF_PARAMS.make),
    model: values(params, EXIF_PARAMS.model),
    lensModel: values(params, EXIF_PARAMS.lensModel),
    country: values(params, EXIF_PARAMS.country),
    state: values(params, EXIF_PARAMS.state),
    city: values(params, EXIF_PARAMS.city)
  };
  const unsupported = (Object.keys(filters) as (keyof BrowseFilters)[])
    .filter((key) => !supportsFilter(url, key))
    .map((key) => [key, FILTER_DEFAULTS[key]]);
  return { ...filters, ...Object.fromEntries(unsupported) };
}

export function withFilterParams(url: URL, filters: BrowseFilters): URL {
  const next = new URL(url);
  for (const name of FILTER_PARAMS) next.searchParams.delete(name);
  for (const [name, key] of SCALAR_PARAMS) {
    const value = filters[key];
    if (value !== FILTER_DEFAULTS[key])
      next.searchParams.append(name, value === true ? '1' : String(value));
  }
  for (const key of EXIF_FILTERS) {
    for (const value of filters[key]) next.searchParams.append(EXIF_PARAMS[key], value);
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
