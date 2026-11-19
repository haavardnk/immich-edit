import type { SearchSort, SortDir, SortField } from '#lib/types/search.ts';
import { readFilterParams, withFilterParams } from '#lib/browse/filterParams.ts';
import { isGridSize, type GridSize } from '#lib/browse/gridSizes.ts';
import { SORT_FIELD_LABELS } from '#lib/browse/sortOptions.ts';

export interface SavedViewTarget {
  path: string;
  sort?: SearchSort;
  size?: GridSize;
}

export const SAVED_VIEW_URL_MAX = 2048;

const BASE = 'http://saved-view.invalid';
const ORIGIN = new URL(BASE).origin;
const SORT_PARAM = 'sort';
const SIZE_PARAM = 'size';
const DIRECTIONS: SortDir[] = ['asc', 'desc'];
const SITE_PATH = /^\/(?![/\\])[!-~]*$/;
const FORBIDDEN = /[#\\]/;

function isSortField(value: string): value is SortField {
  return Object.hasOwn(SORT_FIELD_LABELS, value);
}

function parseSort(value: string | null): SearchSort | undefined {
  const [field = '', direction, ...rest] = (value ?? '').split('-');
  if (rest.length > 0 || !isSortField(field)) return undefined;
  const dir = DIRECTIONS.find((d) => d === direction);
  return dir ? { field, direction: dir } : undefined;
}

function pathOf(url: URL): string {
  const search = url.searchParams.toString();
  return search ? `${url.pathname}?${search}` : url.pathname;
}

function withoutViewParams(value: string): URL {
  const url = new URL(value, BASE);
  url.searchParams.delete(SORT_PARAM);
  url.searchParams.delete(SIZE_PARAM);
  return url;
}

function isSitePath(value: string): boolean {
  return value.length <= SAVED_VIEW_URL_MAX && SITE_PATH.test(value) && !FORBIDDEN.test(value);
}

export function isSavedViewUrl(value: string): boolean {
  if (!isSitePath(value)) return false;
  const url = new URL(value, BASE);
  return url.origin === ORIGIN && isSitePath(url.pathname);
}

export function savedViewUrl(path: string, sort: SearchSort | null, size: GridSize): string | null {
  const url = withoutViewParams(path);
  if (sort) url.searchParams.set(SORT_PARAM, `${sort.field}-${sort.direction}`);
  url.searchParams.set(SIZE_PARAM, size);
  const built = pathOf(url);
  return isSavedViewUrl(built) ? built : null;
}

export function parseSavedViewUrl(value: string): SavedViewTarget | null {
  if (!isSavedViewUrl(value)) return null;
  const path = pathOf(withoutViewParams(value));
  if (!isSavedViewUrl(path)) return null;
  const params = new URL(value, BASE).searchParams;
  const sort = parseSort(params.get(SORT_PARAM));
  const size = params.get(SIZE_PARAM);
  return {
    path,
    ...(sort ? { sort } : {}),
    ...(isGridSize(size) ? { size } : {})
  };
}

export function savedViewPath(value: string): string {
  const url = withoutViewParams(value);
  return pathOf(withFilterParams(url, readFilterParams(url)));
}
