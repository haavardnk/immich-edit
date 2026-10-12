import type { BrowseFilter, SearchSort, SortDir, SortField, Visibility } from '$lib/types/search';
import type { LabelColor } from '$lib/stores/labels';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { readStored, writeStored } from '$lib/utils/storage';
import type { RatingFilter } from '$lib/browse/rating';
import type { ExifFilterKey } from '$lib/browse/exifFilters';
import { serverFilter } from '$lib/browse/serverFilter';

export type { SortDir, SortField, Visibility };
export type LabelFilter = 'any' | 'none' | LabelColor;
export type RejectedFilter = 'any' | 'hide' | 'only';
export const REJECTED_FILTERS: RejectedFilter[] = ['any', 'hide', 'only'];
export type SortFamily = 'timeline' | 'collection' | 'edited';

export interface BrowseFilters {
  favoriteOnly: boolean;
  notInAlbum: boolean;
  untagged: boolean;
  noPeople: boolean;
  rating: RatingFilter;
  filename: string;
  minSize: number | null;
  maxSize: number | null;
  visibility: Visibility;
  takenAfter: string;
  takenBefore: string;
  addedAfter: string;
  addedBefore: string;
  rejected: RejectedFilter;
  label: LabelFilter;
  make: string[];
  model: string[];
  lensModel: string[];
  country: string[];
  state: string[];
  city: string[];
}

export type FilterKey = keyof BrowseFilters;

const SORT_KEY = 'immich-edit:browseSort';

const SORT_DEFAULTS: Record<SortFamily, SearchSort> = {
  timeline: { field: 'taken', direction: 'desc' },
  collection: { field: 'taken', direction: 'asc' },
  edited: { field: 'taken', direction: 'asc' }
};

const SORT_FIELDS: SortField[] = ['taken', 'added', 'rating', 'size'];
const STRUCTURED_SORT_FIELDS: SortField[] = ['rating', 'size'];

function storedSort(value: unknown, fallback: SearchSort): SearchSort {
  if (value === 'asc' || value === 'desc') return { field: 'taken', direction: value };
  if (typeof value !== 'object' || value === null) return fallback;
  const { field, direction } = value as Partial<Record<keyof SearchSort, unknown>>;
  return {
    field: SORT_FIELDS.find((f) => f === field) ?? fallback.field,
    direction: direction === 'asc' || direction === 'desc' ? direction : fallback.direction
  };
}

export const FILTER_DEFAULTS: BrowseFilters = {
  favoriteOnly: false,
  notInAlbum: false,
  untagged: false,
  noPeople: false,
  rating: 'any',
  filename: '',
  minSize: null,
  maxSize: null,
  visibility: 'timeline',
  takenAfter: '',
  takenBefore: '',
  addedAfter: '',
  addedBefore: '',
  rejected: 'any',
  label: 'any',
  make: [],
  model: [],
  lensModel: [],
  country: [],
  state: [],
  city: []
};

export function isDefaultFilter(filters: BrowseFilters, key: keyof BrowseFilters): boolean {
  return JSON.stringify(filters[key]) === JSON.stringify(FILTER_DEFAULTS[key]);
}

export class BrowseControlsStore {
  filters = $state<BrowseFilters>({ ...FILTER_DEFAULTS });
  private sortByFamily = $state<Record<SortFamily, SearchSort>>({ ...SORT_DEFAULTS });
  private family = $state<SortFamily>('timeline');

  constructor() {
    const stored = readStored<Record<string, unknown>>(SORT_KEY);
    if (!stored) return;
    const merged = { ...SORT_DEFAULTS };
    for (const family of Object.keys(SORT_DEFAULTS) as SortFamily[]) {
      merged[family] = storedSort(stored[family], SORT_DEFAULTS[family]);
    }
    this.sortByFamily = merged;
  }

  private get current(): SearchSort {
    const sort = this.sortByFamily[this.family];
    if (STRUCTURED_SORT_FIELDS.includes(sort.field) && !immichCapabilities.structuredSearch) {
      return SORT_DEFAULTS[this.family];
    }
    return sort;
  }

  get sortDir(): SortDir {
    return this.current.direction;
  }

  get sortField(): SortField {
    return this.current.field;
  }

  get sortFamily(): SortFamily {
    return this.family;
  }

  setSortDir(direction: SortDir): void {
    this.setSort({ ...this.sort, direction });
  }

  setSortField(field: SortField): void {
    this.setSort({ ...this.sort, field });
  }

  private setSort(sort: SearchSort): void {
    const next = { ...this.sortByFamily };
    next[this.family] = sort;
    this.sortByFamily = next;
    writeStored(SORT_KEY, next);
  }

  get isDefault(): boolean {
    return (
      this.sortDir === SORT_DEFAULTS[this.family].direction &&
      this.sortField === SORT_DEFAULTS[this.family].field &&
      !this.isFiltered
    );
  }

  get isFiltered(): boolean {
    return (Object.keys(FILTER_DEFAULTS) as (keyof BrowseFilters)[]).some(
      (key) => !isDefaultFilter(this.filters, key)
    );
  }

  private corrected = false;

  reset(): void {
    this.resetFilters();
    this.setSort(SORT_DEFAULTS[this.family]);
  }

  enter(family: SortFamily | null, filters: BrowseFilters): void {
    if (family) this.family = family;
    this.applyFilters(filters);
  }

  prune(key: ExifFilterKey, available: string[]): void {
    const kept = this.filters[key].filter((value) => available.includes(value));
    if (kept.length === this.filters[key].length) return;
    this.corrected = true;
    this.filters[key] = kept;
  }

  takeCorrected(): boolean {
    const corrected = this.corrected;
    this.corrected = false;
    return corrected;
  }

  applyFilters(filters: BrowseFilters): void {
    this.filters = { ...filters };
  }

  private resetFilters(): void {
    this.applyFilters(FILTER_DEFAULTS);
  }

  filter(base: BrowseFilter, flatOnly = false): BrowseFilter {
    return serverFilter(this.filters, base, immichCapabilities.structuredSearch && !flatOnly);
  }

  get sort(): SearchSort {
    return { ...this.current };
  }

  serverFilterKey(flatOnly = false): string {
    return [this.sortField, this.sortDir, JSON.stringify(this.filter({}, flatOnly))].join('|');
  }
}

export const browseControls = new BrowseControlsStore();
