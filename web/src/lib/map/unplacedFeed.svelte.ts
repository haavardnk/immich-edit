import type { MarkerFilter } from '#lib/api/map.ts';
import { searchMetadata } from '#lib/api/search.ts';
import { coordinates } from '#lib/browse/info/photoInfo.ts';
import type { WindowEnd } from '#lib/browse/feedWindow.ts';
import { markerQuery } from '#lib/map/mapArea.ts';
import { browsing, type BrowsePager } from '#lib/stores/browsing.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';
import type { BrowseFilter, PageQuery } from '#lib/types/search.ts';
import { errorMessage } from '#lib/utils/errors.ts';

const UNPLACED_PAGE = 120;

type Search = (query: PageQuery) => ReturnType<typeof searchMetadata>;

export function unplacedFilter(filter: MarkerFilter): BrowseFilter {
  const dates = markerQuery(filter);
  return {
    visibility: 'timeline',
    country: [null],
    favorite: filter.favorite || undefined,
    takenAfter: dates.after,
    takenBefore: dates.before
  };
}

function unplaced(asset: AssetSummary): boolean {
  return !asset.exifInfo || coordinates(asset.exifInfo) === null;
}

export class UnplacedFeed implements BrowsePager {
  assets = $state<AssetSummary[]>([]);
  error = $state<string | null>(null);
  loaded = $state(false);
  readonly hasPrevious = false;
  #filter: MarkerFilter | null = null;
  #cursor = $state<string | null>(null);
  #started = $state(false);
  #loading = $state(false);
  #generation = 0;
  #search: Search;

  constructor(search: Search = searchMetadata) {
    this.#search = search;
  }

  get hasMore(): boolean {
    return !this.#started || this.#cursor !== null;
  }

  get loadingMore(): boolean {
    return this.#loading;
  }

  attach(): void {
    if (!browsing.isFeed(this.assets, this)) browsing.set(this.assets, this);
    browsing.total = this.hasMore ? undefined : this.assets.length;
  }

  show(filter: MarkerFilter): Promise<boolean> {
    if (this.#filter && JSON.stringify(this.#filter) === JSON.stringify(filter)) {
      this.attach();
      return Promise.resolve(false);
    }
    this.#generation += 1;
    this.#filter = filter;
    this.#cursor = null;
    this.#started = false;
    this.#loading = false;
    this.loaded = false;
    this.error = null;
    this.assets = [];
    this.attach();
    return this.loadMore();
  }

  remove(ids: readonly string[]): void {
    const gone = new Set(ids);
    if (!this.assets.some((asset) => gone.has(asset.id))) return;
    this.assets = this.assets.filter((asset) => !gone.has(asset.id));
    this.attach();
  }

  async loadMore(): Promise<boolean> {
    const filter = this.#filter;
    if (!filter || !this.hasMore || this.#loading) return false;
    const generation = this.#generation;
    this.#loading = true;
    try {
      const page = await this.#search({
        filter: unplacedFilter(filter),
        sort: { field: 'taken', direction: 'desc' },
        size: UNPLACED_PAGE,
        withExif: true,
        ...(this.#cursor ? { cursor: this.#cursor } : {})
      });
      if (generation !== this.#generation) return false;
      this.#started = true;
      this.#cursor = page.nextCursor;
      this.assets.push(...page.items.filter(unplaced));
      this.loaded = true;
      if (!this.hasMore) browsing.total = this.assets.length;
      return true;
    } catch (error) {
      if (generation === this.#generation) this.error = errorMessage(error);
      return false;
    } finally {
      if (generation === this.#generation) this.#loading = false;
    }
  }

  loadPrevious(): Promise<boolean> {
    return Promise.resolve(false);
  }

  async loadEdge(end: WindowEnd): Promise<boolean> {
    if (end !== 'last') return false;
    let moved = false;
    while (this.hasMore && (await this.loadMore())) moved = true;
    return moved;
  }

  async jumpTo(assetId: string): Promise<boolean> {
    while (!this.assets.some((asset) => asset.id === assetId)) {
      if (!(await this.loadMore())) return false;
    }
    return true;
  }
}
