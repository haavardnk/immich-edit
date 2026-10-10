import { BROWSE_PAGE_SIZE, searchMetadata, searchStatistics } from '$lib/api/search';
import type { BrowseFilter, SearchPage } from '$lib/types/search';
import { browsing, type BrowsePager } from './browsing.svelte';
import { browseControls } from './browseControls.svelte';
import { loadCullTags, stampCullTags } from './cullTags';
import { selection } from './selection.svelte';
import { toasts } from './toasts.svelte';
import type { AssetSummary } from '$lib/types/album';

export interface BrowseFeedOptions {
  baseFilter: () => BrowseFilter;
  includeStats?: boolean;
  fetcher?: (filter: BrowseFilter, cursor: string | undefined) => Promise<SearchPage>;
  onFetchError?: (initial: boolean, error: unknown) => void;
}

function fetchMetadata(filter: BrowseFilter, cursor: string | undefined): Promise<SearchPage> {
  return searchMetadata({
    filter,
    sort: browseControls.sort,
    size: BROWSE_PAGE_SIZE,
    withExif: true,
    cursor
  });
}

export class BrowseFeed implements BrowsePager {
  assets = $state<AssetSummary[]>([]);
  loading = $state(false);
  loadedOnce = $state(false);
  loadingMore = $state(false);
  nextCursor = $state<string | null>(null);
  totalCount = $state<number | undefined>(undefined);
  private prevKey = '';
  private reqId = 0;
  private loadingAll = false;
  private pendingMore: Promise<boolean> | null = null;
  private opts: BrowseFeedOptions;

  constructor(opts: BrowseFeedOptions) {
    this.opts = opts;
  }

  get hasMore(): boolean {
    return this.nextCursor !== null;
  }

  reset(): void {
    this.reqId += 1;
    this.assets = [];
    this.loadedOnce = false;
    this.loading = false;
    this.loadingMore = false;
    this.nextCursor = null;
    this.totalCount = undefined;
    this.prevKey = '';
    this.pendingMore = null;
  }

  fetchPage(initial: boolean): Promise<boolean> {
    const req = (this.reqId += 1);
    const filter = browseControls.filter(this.opts.baseFilter());
    const cursor = initial ? undefined : (this.nextCursor ?? undefined);
    if (initial) {
      if (!this.loadedOnce) this.loading = true;
      this.nextCursor = null;
      this.totalCount = undefined;
      browsing.total = undefined;
      if (this.opts.includeStats !== false) {
        searchStatistics(filter)
          .then((s) => {
            if (req !== this.reqId) return;
            this.totalCount = s.total;
            browsing.total = s.total;
          })
          .catch((e) => {
            if (req === this.reqId) toasts.fail('stats', e);
          });
      }
    }
    const fetcher = this.opts.fetcher ?? fetchMetadata;
    return Promise.all([fetcher(filter, cursor), loadCullTags().catch(() => undefined)])
      .then(([result]) => {
        if (req !== this.reqId) return false;
        const items = stampCullTags(result.items);
        this.assets = initial ? items : [...this.assets, ...items];
        browsing.set(this.assets, this);
        this.nextCursor = result.nextCursor;
        return true;
      })
      .catch((e) => {
        if (req !== this.reqId) return false;
        if (this.opts.onFetchError) this.opts.onFetchError(initial, e);
        else toasts.fail('load', e);
        return false;
      })
      .finally(() => {
        if (req !== this.reqId) return;
        this.loading = false;
        this.loadedOnce = true;
        if (!this.loadingAll) this.loadingMore = false;
      });
  }

  loadMore(): Promise<boolean> {
    if (this.pendingMore) return this.pendingMore;
    if (this.loadingMore || !this.nextCursor) return Promise.resolve(false);
    this.loadingMore = true;
    const pending = this.fetchPage(false).finally(() => {
      if (this.pendingMore === pending) this.pendingMore = null;
    });
    this.pendingMore = pending;
    return pending;
  }

  async loadAll(): Promise<boolean> {
    if (this.loadingMore || this.loadingAll) return false;
    this.loadingAll = true;
    this.loadingMore = true;
    try {
      while (this.nextCursor) {
        if (!(await this.fetchPage(false))) return false;
      }
      return true;
    } finally {
      this.loadingAll = false;
      this.loadingMore = false;
    }
  }

  watchFilterChange(): void {
    const key = browseControls.serverFilterKey;
    if (this.prevKey && key !== this.prevKey) {
      selection.clear();
      void this.fetchPage(true);
    }
    this.prevKey = key;
  }
}
