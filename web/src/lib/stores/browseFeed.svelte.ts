import { searchMetadata, searchStatistics } from '$lib/api/search';
import { browsing } from './browsing.svelte';
import { browseControls } from './browseControls.svelte';
import { toasts } from './toasts.svelte';
import type { AssetSummary } from '$lib/types/album';

export interface BrowseFeedOptions {
  baseBody: () => Record<string, unknown>;
  includeStats?: boolean;
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
  private opts: BrowseFeedOptions;

  constructor(opts: BrowseFeedOptions) {
    this.opts = opts;
  }

  reset(): void {
    this.assets = [];
    this.loadedOnce = false;
    this.loading = false;
    this.loadingMore = false;
    this.nextPage = null;
    this.totalCount = undefined;
    this.prevKey = '';
  }

  fetchPage(initial: boolean): void {
    const base = this.opts.baseBody();
    if (initial) {
      if (!this.loadedOnce) this.loading = true;
      this.nextPage = null;
      this.totalCount = undefined;
      if (this.opts.includeStats !== false) {
        searchStatistics(browseControls.statsBody(base))
          .then((s) => (this.totalCount = s.total))
          .catch((e) => toasts.push('error', `stats: ${(e as Error).message}`));
      }
    }
    const body = browseControls.searchBody(base);
    if (!initial && this.nextPage) body.page = this.nextPage;
    searchMetadata(body)
      .then((result) => {
        this.assets = initial ? result.items : [...this.assets, ...result.items];
        browsing.set(this.assets);
        this.nextPage = result.nextPage;
      })
      .catch((e) => {
        if (this.opts.onFetchError) this.opts.onFetchError(initial, e);
        else toasts.push('error', `load: ${(e as Error).message}`);
      })
      .finally(() => {
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
      this.fetchPage(true);
    }
    this.prevKey = key;
  }
}
