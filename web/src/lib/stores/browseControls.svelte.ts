import type { SearchQuery, SortDir, Visibility } from '$lib/types/search';
import { readStored, writeStored } from '$lib/utils/storage';

export type { SortDir, Visibility };
export type RatingFilter = 'any' | 'unrated' | 1 | 2 | 3 | 4 | 5;
export type SortFamily = 'timeline' | 'collection' | 'edited';

export interface BrowseFilters {
  favoriteOnly: boolean;
  rating: RatingFilter;
  filename: string;
  visibility: Visibility;
  takenAfter: string;
  takenBefore: string;
  excludeRejected: boolean;
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
  excludeRejected: false
};

export class BrowseControlsStore {
  favoriteOnly = $state(false);
  rating = $state<RatingFilter>('any');
  filename = $state('');
  visibility = $state<Visibility>('timeline');
  takenAfter = $state('');
  takenBefore = $state('');
  excludeRejected = $state(false);
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
      !this.excludeRejected
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
      this.excludeRejected
    );
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
      excludeRejected: this.excludeRejected
    };
  }

  applyFilters(filters: BrowseFilters): void {
    this.favoriteOnly = filters.favoriteOnly;
    this.rating = filters.rating;
    this.filename = filters.filename;
    this.visibility = filters.visibility;
    this.takenAfter = filters.takenAfter;
    this.takenBefore = filters.takenBefore;
    this.excludeRejected = filters.excludeRejected;
  }

  private resetFilters(): void {
    this.applyFilters(FILTER_DEFAULTS);
  }

  searchBody(base: SearchQuery): SearchQuery {
    const body: SearchQuery = {
      ...base,
      withExif: true,
      size: 500,
      order: this.sortDir,
      visibility: this.visibility,
      type: 'IMAGE'
    };
    if (this.favoriteOnly && base.isFavorite === undefined) {
      body.isFavorite = true;
    }
    if (typeof this.rating === 'number') {
      body.rating = this.rating;
    } else if (this.rating === 'unrated') {
      body.rating = null;
    }
    if (this.filename) {
      body.originalFileName = this.filename;
    }
    if (this.takenAfter) {
      body.takenAfter = new Date(this.takenAfter).toISOString();
    }
    if (this.takenBefore) {
      body.takenBefore = new Date(this.takenBefore + 'T23:59:59').toISOString();
    }
    return body;
  }

  statsBody(base: SearchQuery): SearchQuery {
    const body = this.searchBody(base);
    delete body.size;
    delete body.order;
    delete body.withExif;
    delete body.page;
    return body;
  }

  smartSearchBody(base: SearchQuery): SearchQuery {
    const body: SearchQuery = {
      ...base,
      withExif: true,
      size: 500,
      visibility: this.visibility,
      type: 'IMAGE'
    };
    if (this.favoriteOnly && base.isFavorite === undefined) {
      body.isFavorite = true;
    }
    if (typeof this.rating === 'number') {
      body.rating = this.rating;
    } else if (this.rating === 'unrated') {
      body.rating = null;
    }
    if (this.takenAfter) {
      body.takenAfter = new Date(this.takenAfter).toISOString();
    }
    if (this.takenBefore) {
      body.takenBefore = new Date(this.takenBefore + 'T23:59:59').toISOString();
    }
    return body;
  }

  get serverFilterKey(): string {
    return [
      this.sortDir,
      this.favoriteOnly,
      this.rating,
      this.filename,
      this.visibility,
      this.takenAfter,
      this.takenBefore
    ].join('|');
  }
}

export const browseControls = new BrowseControlsStore();
