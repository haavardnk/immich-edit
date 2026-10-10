import type { BrowseFilter, SearchSort, SortDir, Visibility } from '$lib/types/search';
import type { AssetSummary } from '$lib/types/album';
import { labelOf, type LabelColor } from '$lib/stores/labels';
import { isRejected } from '$lib/browse/reject';
import { immichCapabilities } from '$lib/stores/immichCapabilities.svelte';
import { readStored, writeStored } from '$lib/utils/storage';

export type { SortDir, Visibility };
export type MinRating = '1+' | '2+' | '3+' | '4+';
export const MIN_RATINGS: MinRating[] = ['1+', '2+', '3+', '4+'];
export type RatingFilter = 'any' | 'unrated' | 1 | 2 | 3 | 4 | 5 | MinRating;
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

const SORT_DEFAULTS: Record<SortFamily, SortDir> = {
  timeline: 'desc',
  collection: 'asc',
  edited: 'asc'
};

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
  private sortByFamily = $state<Record<SortFamily, SortDir>>({ ...SORT_DEFAULTS });
  private family = $state<SortFamily>('timeline');

  constructor() {
    const stored = readStored<Record<SortFamily, SortDir>>(SORT_KEY);
    if (!stored) return;
    const merged = { ...SORT_DEFAULTS };
    for (const family of Object.keys(SORT_DEFAULTS) as SortFamily[]) {
      const dir = stored[family];
      if (dir === 'asc' || dir === 'desc') merged[family] = dir;
    }
    this.sortByFamily = merged;
  }

  get sortDir(): SortDir {
    return this.sortByFamily[this.family];
  }

  get sortFamily(): SortFamily {
    return this.family;
  }

  setSortDir(dir: SortDir): void {
    const next = { ...this.sortByFamily };
    next[this.family] = dir;
    this.sortByFamily = next;
    writeStored(SORT_KEY, next);
  }

  get isDefault(): boolean {
    return (
      this.sortDir === SORT_DEFAULTS[this.family] &&
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
    const min = MIN_RATINGS.find((rating) => rating === this.rating);
    if (min && (asset.exifInfo?.rating ?? 0) < parseInt(min, 10)) return true;
    if (this.rejected !== 'any' && isRejected(asset) !== (this.rejected === 'only')) return true;
    if (this.label === 'any') return false;
    return labelOf(asset) !== (this.label === 'none' ? null : this.label);
  }

  private context: string | null = null;

  reset(): void {
    this.resetFilters();
    this.setSortDir(SORT_DEFAULTS[this.family]);
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
    const rating = this.serverRating(flatOnly);
    if (this.favoriteOnly) filter.favorite = true;
    if (typeof rating === 'number') filter.rating = { eq: rating };
    else if (rating === 'unrated') filter.rating = 'unrated';
    else if (rating !== 'any') filter.rating = { gte: parseInt(rating, 10) };
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
    const min = MIN_RATINGS.some((rating) => rating === this.rating);
    if (min && !this.structured(flatOnly)) return 'any';
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
    return { field: 'taken', direction: this.sortDir };
  }

  serverFilterKey(flatOnly = false): string {
    return [
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
