import type { MockTag, MockAssetSummary } from './helpers';

const SEARCH_FILTER_KEYS = new Set([
  'scope',
  'favorite',
  'notInAlbum',
  'untagged',
  'noPeople',
  'albums',
  'people',
  'tags',
  'visibility',
  'imagesOnly',
  'rating',
  'filename',
  'description',
  'path',
  'ocr',
  'minSize',
  'maxSize',
  'takenAfter',
  'takenBefore',
  'addedAfter',
  'addedBefore',
  'rejected',
  'label',
  'make',
  'model',
  'lensModel',
  'country',
  'state',
  'city'
]);

type ExifKey = 'make' | 'model' | 'lensModel' | 'country' | 'state' | 'city';
const EXIF_PARENTS: Record<ExifKey, ExifKey[]> = {
  make: [],
  model: ['make'],
  lensModel: ['model', 'make'],
  country: [],
  state: ['country'],
  city: ['state', 'country']
};
const EXIF_KEYS = Object.keys(EXIF_PARENTS) as ExifKey[];
export type ExifFilter = Partial<Record<ExifKey, (string | null)[]>>;

const SEARCH_BODY_KEYS: Record<string, Set<string>> = {
  '/api/search/metadata': new Set(['filter', 'sort', 'size', 'cursor', 'withExif']),
  '/api/search/smart': new Set(['text', 'filter', 'size', 'cursor', 'withExif']),
  '/api/search/statistics': new Set(['filter']),
  '/api/search/window': new Set(['assetId', 'radius', 'filter', 'sort']),
  '/api/search/suggestions': new Set(['field', 'make', 'model', 'country', 'state'])
};

export function searchShapeProblem(
  path: string,
  body: Record<string, unknown>,
  structured: boolean
): string | null {
  const allowed = SEARCH_BODY_KEYS[path];
  if (!allowed) return null;
  const filter = (body.filter ?? {}) as Record<string, unknown>;
  const unknown = [
    ...Object.keys(body).filter((key) => !allowed.has(key)),
    ...Object.keys(filter)
      .filter((key) => !SEARCH_FILTER_KEYS.has(key))
      .map((key) => `filter.${key}`)
  ];
  if (unknown.length) return `unknown search fields: ${unknown.join(', ')}`;
  if (
    path === '/api/search/smart' &&
    ['filename', 'description', 'path'].some((k) => k in filter)
  ) {
    return 'smart search cannot filter by text fields';
  }
  const flat = path === '/api/search/smart' || !structured;
  const range =
    typeof filter.rating === 'object' &&
    filter.rating !== null &&
    ('gte' in filter.rating || 'lte' in filter.rating);
  if (flat && (filter.rejected === 'hide' || filter.label === 'none' || range)) {
    return 'flat search cannot exclude tags or match a rating range';
  }
  if (flat && filter.noPeople) {
    return 'flat search cannot find photos without people';
  }
  const picked = ['albums', 'people', 'tags'].map((key) => filter[key] as IdSet | undefined);
  if (flat && picked.some((set) => set && (set.any || set.none))) {
    return 'flat search can only match all of the picked ids';
  }
  if (flat && (filter.minSize !== undefined || filter.maxSize !== undefined)) {
    return 'flat search cannot filter by file size';
  }
  const sortField = (body.sort as { field?: unknown } | undefined)?.field;
  if (flat && (sortField === 'rating' || sortField === 'size')) {
    return 'flat search cannot sort by rating or size';
  }
  if (flat && EXIF_KEYS.some((key) => ((filter as ExifFilter)[key]?.length ?? 0) > 1)) {
    return 'flat search cannot match several exif values';
  }
  return null;
}

export function matchesExif(asset: MockAssetSummary, filter: ExifFilter): boolean {
  return EXIF_KEYS.every(
    (key) => !filter[key] || filter[key].includes(asset.exifInfo?.[key] ?? null)
  );
}

export function suggestions(
  assets: MockAssetSummary[],
  query: { field: ExifKey } & ExifFilter
): (string | null)[] {
  const parent = EXIF_PARENTS[query.field].find((key) => query[key]?.length);
  const values = assets
    .filter((a) => !parent || (query[parent] ?? []).includes(a.exifInfo?.[parent] ?? ''))
    .map((a) => a.exifInfo?.[query.field] || null);
  const known = values.filter((value): value is string => value !== null);
  return [...[...new Set(known)].sort(), ...(values.includes(null) ? [null] : [])];
}

export type ManagedFilter = { rejected?: string; label?: string };
export type IdSet = { all?: string[]; any?: string[]; none?: string[] };

export interface MockPerson {
  id: string;
  name: string;
  isHidden?: boolean;
  assetIds: string[];
}

export interface MockAlbum {
  id: string;
  albumName: string;
  assetIds: string[];
}

export function matchesIds(ids: string[], set: IdSet | undefined): boolean {
  if (!set) return true;
  return (
    (set.all ?? []).every((id) => ids.includes(id)) &&
    (!set.any || set.any.some((id) => ids.includes(id))) &&
    !(set.none ?? []).some((id) => ids.includes(id))
  );
}

export function matchesManagedTags(tags: MockTag[], filter: ManagedFilter): boolean {
  const values = tags.map((t) => t.value.toLowerCase());
  const rejected = values.includes('immich-edit/reject');
  if (filter.rejected && rejected !== (filter.rejected === 'only')) return false;
  if (!filter.label) return true;
  if (filter.label === 'none') return !values.some((v) => v.startsWith('immich-edit/label/'));
  return values.includes(`immich-edit/label/${filter.label}`);
}
