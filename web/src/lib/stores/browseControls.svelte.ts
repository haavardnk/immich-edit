import type { BrowseFilter, SearchSort, SortDir, SortField, Visibility } from '$lib/types/search';
import type { AssetSummary } from '$lib/types/album';
import { labelOf, type LabelColor } from '$lib/stores/labels';
import { isRejected } from '$lib/browse/reject';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { readStored, writeStored } from '$lib/utils/storage';
import { isRange, ratingHides, ratingMatch, type RatingFilter } from '$lib/browse/rating';
import { EXIF_FILTERS, UNKNOWN_EXIF, type ExifFilterKey } from '$lib/browse/exifFilters';
import { toBytes } from '$lib/browse/fileSize';

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
  rejected: RejectedFilter;
  label: LabelFilter;
  make: string[];
  model: string[];
  lensModel: string[];
  country: string[];
  state: string[];
  city: string[];
}

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
  rejected: 'any',
  label: 'any',
  make: [],
  model: [],
  lensModel: [],
  country: [],
  state: [],
  city: []
};

export class BrowseControlsStore {
  favoriteOnly = $state(false);
  notInAlbum = $state(false);
  untagged = $state(false);
  noPeople = $state(false);
  rating = $state<RatingFilter>('any');
  filename = $state('');
  minSize = $state<number | null>(null);
  maxSize = $state<number | null>(null);
  visibility = $state<Visibility>('timeline');
  takenAfter = $state('');
  takenBefore = $state('');
  rejected = $state<RejectedFilter>('any');
  label = $state<LabelFilter>('any');
  make = $state<string[]>([]);
  model = $state<string[]>([]);
  lensModel = $state<string[]>([]);
  country = $state<string[]>([]);
  state = $state<string[]>([]);
  city = $state<string[]>([]);
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
    const filters = this.filters;
    return (Object.keys(FILTER_DEFAULTS) as (keyof BrowseFilters)[]).some((key) => {
      const value = filters[key];
      return Array.isArray(value) ? value.length > 0 : value !== FILTER_DEFAULTS[key];
    });
  }

  hides(asset: AssetSummary): boolean {
    if (ratingHides(this.rating, asset.exifInfo?.rating)) return true;
    if (this.rejected !== 'any' && isRejected(asset) !== (this.rejected === 'only')) return true;
    const exif = asset.exifInfo;
    if (EXIF_FILTERS.some((key) => this[key].length > 0 && !this[key].includes(exif?.[key] ?? '')))
      return true;
    if (this.label === 'any') return false;
    return labelOf(asset) !== (this.label === 'none' ? null : this.label);
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
    const kept = this[key].filter((value) => available.includes(value));
    if (kept.length === this[key].length) return;
    this.corrected = true;
    this[key] = kept;
  }

  takeCorrected(): boolean {
    const corrected = this.corrected;
    this.corrected = false;
    return corrected;
  }

  get filters(): BrowseFilters {
    return {
      favoriteOnly: this.favoriteOnly,
      notInAlbum: this.notInAlbum,
      untagged: this.untagged,
      noPeople: this.noPeople,
      rating: this.rating,
      filename: this.filename,
      minSize: this.minSize,
      maxSize: this.maxSize,
      visibility: this.visibility,
      takenAfter: this.takenAfter,
      takenBefore: this.takenBefore,
      rejected: this.rejected,
      label: this.label,
      make: this.make,
      model: this.model,
      lensModel: this.lensModel,
      country: this.country,
      state: this.state,
      city: this.city
    };
  }

  applyFilters(filters: BrowseFilters): void {
    this.favoriteOnly = filters.favoriteOnly;
    this.notInAlbum = filters.notInAlbum;
    this.untagged = filters.untagged;
    this.noPeople = filters.noPeople;
    this.rating = filters.rating;
    this.filename = filters.filename;
    this.minSize = filters.minSize;
    this.maxSize = filters.maxSize;
    this.visibility = filters.visibility;
    this.takenAfter = filters.takenAfter;
    this.takenBefore = filters.takenBefore;
    this.rejected = filters.rejected;
    this.label = filters.label;
    this.make = filters.make;
    this.model = filters.model;
    this.lensModel = filters.lensModel;
    this.country = filters.country;
    this.state = filters.state;
    this.city = filters.city;
  }

  private resetFilters(): void {
    this.applyFilters(FILTER_DEFAULTS);
  }

  filter(base: BrowseFilter, flatOnly = false): BrowseFilter {
    const filter: BrowseFilter = { ...base, visibility: this.visibility, imagesOnly: true };
    const rating = ratingMatch(this.serverRating(flatOnly));
    if (this.favoriteOnly) filter.favorite = true;
    if (this.notInAlbum) filter.notInAlbum = true;
    if (this.untagged) filter.untagged = true;
    if (this.noPeople) filter.noPeople = true;
    if (rating) filter.rating = rating;
    if (this.filename) filter.filename = this.filename;
    if (this.minSize !== null) filter.minSize = toBytes(this.minSize);
    if (this.maxSize !== null) filter.maxSize = toBytes(this.maxSize);
    if (this.takenAfter) filter.takenAfter = new Date(this.takenAfter).toISOString();
    if (this.takenBefore) {
      filter.takenBefore = new Date(this.takenBefore + 'T23:59:59').toISOString();
    }
    const rejected = this.serverRejected(flatOnly);
    const label = this.serverLabel(flatOnly);
    if (rejected !== 'any') filter.rejected = rejected;
    if (label !== 'any') filter.label = label;
    for (const key of EXIF_FILTERS) {
      const values = this.serverValues(key, flatOnly);
      if (values.length > 0)
        filter[key] = values.map((value) => (value === UNKNOWN_EXIF ? null : value));
    }
    return filter;
  }

  private structured(flatOnly: boolean): boolean {
    return immichCapabilities.structuredSearch && !flatOnly;
  }

  private serverValues(key: ExifFilterKey, flatOnly: boolean): string[] {
    if (this[key].length > 1 && !this.structured(flatOnly)) return [];
    return this[key];
  }

  private serverRating(flatOnly: boolean): RatingFilter {
    if (isRange(this.rating) && !this.structured(flatOnly)) return 'any';
    return this.rating;
  }

  private serverLabel(flatOnly: boolean): LabelFilter {
    if (this.label === 'none' && !this.structured(flatOnly)) return 'any';
    return this.label;
  }

  private serverRejected(flatOnly: boolean): RejectedFilter {
    if (this.rejected === 'hide' && !this.structured(flatOnly)) return 'any';
    return this.rejected;
  }

  get sort(): SearchSort {
    return { ...this.current };
  }

  serverFilterKey(flatOnly = false): string {
    return [
      this.sortField,
      this.sortDir,
      this.favoriteOnly,
      this.notInAlbum,
      this.untagged,
      this.noPeople,
      this.serverRating(flatOnly),
      this.filename,
      this.minSize,
      this.maxSize,
      this.visibility,
      this.takenAfter,
      this.takenBefore,
      this.serverRejected(flatOnly),
      this.serverLabel(flatOnly),
      JSON.stringify(EXIF_FILTERS.map((key) => this.serverValues(key, flatOnly)))
    ].join('|');
  }
}

export const browseControls = new BrowseControlsStore();
