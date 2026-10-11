import type { BrowseFilter, SearchSort, SortDir, SortField, Visibility } from '$lib/types/search';
import type { AssetSummary } from '$lib/types/album';
import { labelOf, type LabelColor } from '$lib/stores/labels';
import { isRejected } from '$lib/browse/reject';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { readStored, writeStored } from '$lib/utils/storage';
import { isRange, ratingHides, ratingMatch, type RatingFilter } from '$lib/browse/rating';

export type { SortDir, SortField, Visibility };
export type LabelFilter = 'any' | 'none' | LabelColor;
export type RejectedFilter = 'any' | 'hide' | 'only';
export const REJECTED_FILTERS: RejectedFilter[] = ['any', 'hide', 'only'];
export type SortFamily = 'timeline' | 'collection' | 'edited';

export interface BrowseFilters {
  favoriteOnly: boolean;
  rating: RatingFilter;
  filename: string;
  visibility: Visibility;
  takenAfter: string;
  takenBefore: string;
  rejected: RejectedFilter;
  label: LabelFilter;
}

const SORT_KEY = 'immich-edit:browseSort';

const SORT_DEFAULTS: Record<SortFamily, SearchSort> = {
  timeline: { field: 'taken', direction: 'desc' },
  collection: { field: 'taken', direction: 'asc' },
  edited: { field: 'taken', direction: 'asc' }
};

function storedSort(value: unknown, fallback: SearchSort): SearchSort {
  if (value === 'asc' || value === 'desc') return { field: 'taken', direction: value };
  if (typeof value !== 'object' || value === null) return fallback;
  const { field, direction } = value as Partial<Record<keyof SearchSort, unknown>>;
  return {
    field: field === 'taken' || field === 'added' || field === 'rating' ? field : fallback.field,
    direction: direction === 'asc' || direction === 'desc' ? direction : fallback.direction
  };
}

export const FILTER_DEFAULTS: BrowseFilters = {
  favoriteOnly: false,
  rating: 'any',
  filename: '',
  visibility: 'timeline',
  takenAfter: '',
  takenBefore: '',
  rejected: 'any',
  label: 'any'
};

export class BrowseControlsStore {
  favoriteOnly = $state(false);
  rating = $state<RatingFilter>('any');
  filename = $state('');
  visibility = $state<Visibility>('timeline');
  takenAfter = $state('');
  takenBefore = $state('');
  rejected = $state<RejectedFilter>('any');
  label = $state<LabelFilter>('any');
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
    if (sort.field === 'rating' && !immichCapabilities.structuredSearch) {
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
      !this.favoriteOnly &&
      this.rating === 'any' &&
      this.filename === '' &&
      this.visibility === 'timeline' &&
      this.takenAfter === '' &&
      this.takenBefore === '' &&
      this.rejected === 'any' &&
      this.label === 'any'
    );
  }

  get isFiltered(): boolean {
    return (
      this.favoriteOnly ||
      this.rating !== 'any' ||
      this.filename !== '' ||
      this.visibility !== 'timeline' ||
      this.takenAfter !== '' ||
      this.takenBefore !== '' ||
      this.rejected !== 'any' ||
      this.label !== 'any'
    );
  }

  hides(asset: AssetSummary): boolean {
    if (ratingHides(this.rating, asset.exifInfo?.rating)) return true;
    if (this.rejected !== 'any' && isRejected(asset) !== (this.rejected === 'only')) return true;
    if (this.label === 'any') return false;
    return labelOf(asset) !== (this.label === 'none' ? null : this.label);
  }

  private context: string | null = null;

  reset(): void {
    this.resetFilters();
    this.setSort(SORT_DEFAULTS[this.family]);
  }

  enter(context: string, family: SortFamily | null): void {
    if (family) this.family = family;
    if (this.context === context) return;
    this.context = context;
    this.resetFilters();
  }

  get filters(): BrowseFilters {
    return {
      favoriteOnly: this.favoriteOnly,
      rating: this.rating,
      filename: this.filename,
      visibility: this.visibility,
      takenAfter: this.takenAfter,
      takenBefore: this.takenBefore,
      rejected: this.rejected,
      label: this.label
    };
  }

  applyFilters(filters: BrowseFilters): void {
    this.favoriteOnly = filters.favoriteOnly;
    this.rating = filters.rating;
    this.filename = filters.filename;
    this.visibility = filters.visibility;
    this.takenAfter = filters.takenAfter;
    this.takenBefore = filters.takenBefore;
    this.rejected = filters.rejected;
    this.label = filters.label;
  }

  private resetFilters(): void {
    this.applyFilters(FILTER_DEFAULTS);
  }

  filter(base: BrowseFilter, flatOnly = false): BrowseFilter {
    const filter: BrowseFilter = { ...base, visibility: this.visibility, imagesOnly: true };
    const rating = ratingMatch(this.serverRating(flatOnly));
    if (this.favoriteOnly) filter.favorite = true;
    if (rating) filter.rating = rating;
    if (this.filename) filter.filename = this.filename;
    if (this.takenAfter) filter.takenAfter = new Date(this.takenAfter).toISOString();
    if (this.takenBefore) {
      filter.takenBefore = new Date(this.takenBefore + 'T23:59:59').toISOString();
    }
    const rejected = this.serverRejected(flatOnly);
    const label = this.serverLabel(flatOnly);
    if (rejected !== 'any') filter.rejected = rejected;
    if (label !== 'any') filter.label = label;
    return filter;
  }

  private structured(flatOnly: boolean): boolean {
    return immichCapabilities.structuredSearch && !flatOnly;
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
      this.serverRating(flatOnly),
      this.filename,
      this.visibility,
      this.takenAfter,
      this.takenBefore,
      this.serverRejected(flatOnly),
      this.serverLabel(flatOnly)
    ].join('|');
  }
}

export const browseControls = new BrowseControlsStore();
