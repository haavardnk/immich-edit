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

  fetchPage(initial: boolean): void {
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
    Promise.all([fetcher(body), rejected.load().catch(() => undefined)])
      .then(([result]) => {
        if (req !== this.reqId) return;
        const items = rejected.stamp(result.items);
        this.assets = initial ? items : [...this.assets, ...items];
        browsing.set(this.assets);
        this.nextPage = result.nextPage;
      })
      .catch((e) => {
        if (req !== this.reqId) return;
        if (this.opts.onFetchError) this.opts.onFetchError(initial, e);
        else toasts.fail('load', e);
      })
      .finally(() => {
        if (req !== this.reqId) return;
        this.loading = false;
        this.loadedOnce = true;
        this.loadingMore = false;
      });
  }

  loadMore(): void {
    if (this.loadingMore || !this.nextPage) return;
    this.loadingMore = true;
    this.fetchPage(false);
  }

  watchFilterChange(): void {
    const key = browseControls.serverFilterKey;
    if (this.prevKey && key !== this.prevKey) {
      selection.clear();
      this.fetchPage(true);
    }
    this.prevKey = key;
  }
}
