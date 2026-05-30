export type SortDir = 'asc' | 'desc';
export type RatingFilter = 'any' | 'unrated' | 1 | 2 | 3 | 4 | 5;
export type Visibility = 'timeline' | 'archive' | 'hidden';

class BrowseControlsStore {
  sortDir = $state<SortDir>('desc');
  favoriteOnly = $state(false);
  rating = $state<RatingFilter>('any');
  filename = $state('');
  visibility = $state<Visibility>('timeline');
  takenAfter = $state('');
  takenBefore = $state('');

  get isDefault(): boolean {
    return (
      this.sortDir === 'desc' &&
      !this.favoriteOnly &&
      this.rating === 'any' &&
      this.filename === '' &&
      this.visibility === 'timeline' &&
      this.takenAfter === '' &&
      this.takenBefore === ''
    );
  }

  get isFiltered(): boolean {
    return (
      this.favoriteOnly ||
      this.rating !== 'any' ||
      this.filename !== '' ||
      this.visibility !== 'timeline' ||
      this.takenAfter !== '' ||
      this.takenBefore !== ''
    );
  }

  reset(): void {
    this.sortDir = 'desc';
    this.favoriteOnly = false;
    this.rating = 'any';
    this.filename = '';
    this.visibility = 'timeline';
    this.takenAfter = '';
    this.takenBefore = '';
  }

  searchBody(base: Record<string, unknown>): Record<string, unknown> {
    const body: Record<string, unknown> = {
      ...base,
      withExif: true,
      size: 500,
      order: this.sortDir,
      visibility: this.visibility,
      type: 'IMAGE',
    };
    if (this.favoriteOnly && !('isFavorite' in base)) {
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

  statsBody(base: Record<string, unknown>): Record<string, unknown> {
    const body = this.searchBody(base);
    delete body.size;
    delete body.order;
    delete body.withExif;
    delete body.page;
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
      this.takenBefore,
    ].join('|');
  }
}

export const browseControls = new BrowseControlsStore();

