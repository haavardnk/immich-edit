import { searchMetadata, searchStatistics } from '$lib/api/search';
import type { SearchQuery, SearchResult } from '$lib/types/search';
import { browsing } from './browsing.svelte';
import { browseControls } from './browseControls.svelte';
import { rejected } from './rejected.svelte';
import { selection } from './selection.svelte';
import { toasts } from './toasts.svelte';
import type { AssetSummary } from '$lib/types/album';

export interface BrowseFeedOptions {
  baseBody: () => SearchQuery;
  includeStats?: boolean;
  fetcher?: (body: SearchQuery) => Promise<SearchResult>;
  buildBody?: (base: SearchQuery) => SearchQuery;
  onFetchError?: (initial: boolean, error: unknown) => void;
}

export class BrowseFeed {
  assets = $state<AssetSummary[]>([]);
  loading = $state(false);
  loadedOnce = $state(false);
  loadingMore = $state(false);
  nextPage = $state<string | null>(null);
  totalCount = $state<number | undefined>(undefined);
  private prevKey = '';
  private reqId = 0;
  private loadingAll = false;
  private opts: BrowseFeedOptions;

  constructor(opts: BrowseFeedOptions) {
    this.opts = opts;
  }

  reset(): void {
    this.reqId += 1;
    this.assets = [];
    this.loadedOnce = false;
    this.loading = false;
    this.loadingMore = false;
    this.nextPage = null;
    this.totalCount = undefined;
    this.prevKey = '';
  }

  fetchPage(initial: boolean): Promise<boolean> {
    const req = (this.reqId += 1);
    const base = this.opts.baseBody();
    if (initial) {
      if (!this.loadedOnce) this.loading = true;
      this.nextPage = null;
      this.totalCount = undefined;
      browsing.setContext(browseControls.statsBody(base), undefined);
      if (this.opts.includeStats !== false) {
        searchStatistics(browseControls.statsBody(base))
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
    const body = (this.opts.buildBody ?? browseControls.searchBody.bind(browseControls))(base);
    if (!initial && this.nextPage) body.page = Number(this.nextPage);
    const fetcher = this.opts.fetcher ?? searchMetadata;
    return Promise.all([fetcher(body), rejected.load().catch(() => undefined)])
      .then(([result]) => {
        if (req !== this.reqId) return false;
        const items = rejected.stamp(result.items);
        this.assets = initial ? items : [...this.assets, ...items];
        browsing.set(this.assets);
        this.nextPage = result.nextPage;
        if (this.totalCount === undefined) {
          this.totalCount = result.total;
          browsing.total = result.total;
        }
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

  loadMore(): void {
    if (this.loadingMore || !this.nextPage) return;
    this.loadingMore = true;
    void this.fetchPage(false);
  }

  async loadAll(): Promise<boolean> {
    if (this.loadingMore || this.loadingAll) return false;
    this.loadingAll = true;
    this.loadingMore = true;
    try {
      while (this.nextPage) {
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
