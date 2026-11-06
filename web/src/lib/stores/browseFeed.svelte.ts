import {
  BROWSE_PAGE_SIZE,
  searchFirst,
  searchMetadata,
  searchStatistics,
  searchWindow
} from '#lib/api/search.ts';
import type {
  BrowseFilter,
  SearchPage,
  SearchSort,
  SearchWindow,
  WindowReach
} from '#lib/types/search.ts';
import { browsing, type BrowsePager } from './browsing.svelte';
import { browseControls } from './browseControls.svelte';
import { loadCullTags, stampCullTags } from './cullTags';
import { selection } from './selection.svelte';
import { toasts } from './toasts.svelte';
import type { AssetSummary } from '#lib/types/album.ts';
import {
  mergeWindow,
  singleAnchor,
  windowEdge,
  windowEdges,
  type WindowEnd
} from '#lib/browse/feedWindow.ts';

export const JUMP_REACH: WindowReach = { before: 100, after: 400 };
const EDGE_REACH: WindowReach = { before: 400, after: 0 };
const EXTEND_REACH = BROWSE_PAGE_SIZE;
const EDGE_RETRIES = 3;

type WindowSide = 'before' | 'after';

type JumpOutcome = 'accepted' | 'failed' | 'stale';

export type JumpOptions = { reach?: WindowReach; failure?: string | null };

export interface BrowseFeedOptions {
  baseFilter: () => BrowseFilter;
  flatOnly?: () => boolean;
  includeStats?: boolean;
  fetcher?: (filter: BrowseFilter, cursor: string | undefined) => Promise<SearchPage>;
  windowFetcher?: (
    assetId: string,
    filter: BrowseFilter,
    reach: WindowReach
  ) => Promise<SearchWindow>;
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

function fetchWindow(
  assetId: string,
  filter: BrowseFilter,
  reach: WindowReach
): Promise<SearchWindow> {
  return searchWindow(assetId, filter, browseControls.sort, reach);
}

function reversed(sort: SearchSort): SearchSort {
  return { ...sort, direction: sort.direction === 'desc' ? 'asc' : 'desc' };
}

export class BrowseFeed implements BrowsePager {
  assets = $state<AssetSummary[]>([]);
  loading = $state(false);
  loadedOnce = $state(false);
  loadingMore = $state(false);
  loadingPrevious = $state(false);
  nextCursor = $state<string | null>(null);
  totalCount = $state<number | undefined>(undefined);
  windowed = $state(false);
  #before = $state(false);
  #after = $state(false);
  #jump = 0;
  #countToken = 0;
  #countKey: string | null = null;
  private prevKey = '';
  private reqId = 0;
  private loadingAll = false;
  private pendingMore: Promise<boolean> | null = null;
  private pendingPrevious: Promise<boolean> | null = null;
  private opts: BrowseFeedOptions;

  constructor(opts: BrowseFeedOptions) {
    this.opts = opts;
  }

  get hasMore(): boolean {
    return this.windowed ? this.#after : this.nextCursor !== null;
  }

  get hasPrevious(): boolean {
    return this.windowed && this.#before;
  }

  get metadataBacked(): boolean {
    return !this.opts.fetcher;
  }

  reset(): void {
    this.reqId += 1;
    this.#jump += 1;
    this.#countToken += 1;
    this.#countKey = null;
    this.assets = [];
    this.loadedOnce = false;
    this.loading = false;
    this.loadingMore = false;
    this.nextCursor = null;
    this.totalCount = undefined;
    this.prevKey = '';
    this.leaveWindow();
  }

  currentFilter(): BrowseFilter {
    return browseControls.filter(this.opts.baseFilter(), this.opts.flatOnly?.());
  }

  private serverKey(): string {
    return browseControls.serverFilterKey(this.opts.flatOnly?.());
  }

  private leaveWindow(): void {
    this.windowed = false;
    this.#before = false;
    this.#after = false;
    this.pendingMore = null;
    this.pendingPrevious = null;
    this.loadingPrevious = false;
  }

  private startCount(filter: BrowseFilter): void {
    const token = (this.#countToken += 1);
    this.#countKey = this.serverKey();
    this.totalCount = undefined;
    browsing.total = undefined;
    if (this.opts.includeStats === false) return;
    searchStatistics(filter)
      .then((s) => {
        if (token !== this.#countToken) return;
        this.totalCount = s.total;
        browsing.total = s.total;
      })
      .catch((e) => {
        if (token === this.#countToken) toasts.fail('stats', e);
      });
  }

  private publish(assets: AssetSummary[]): void {
    this.assets = assets;
    browsing.set(this.assets, this);
  }

  fetchPage(initial: boolean): Promise<boolean> {
    const req = (this.reqId += 1);
    const filter = this.currentFilter();
    const cursor = initial ? undefined : (this.nextCursor ?? undefined);
    if (initial) {
      if (!this.loadedOnce) this.loading = true;
      this.nextCursor = null;
      this.leaveWindow();
      this.startCount(filter);
    }
    const fetcher = this.opts.fetcher ?? fetchMetadata;
    const fetchFilled = async (): Promise<SearchPage> => {
      let result = await fetcher(filter, cursor);
      while (!result.items.length && result.nextCursor && req === this.reqId) {
        result = await fetcher(filter, result.nextCursor);
      }
      return result;
    };
    return Promise.all([fetchFilled(), loadCullTags().catch(() => undefined)])
      .then(([result]) => {
        if (req !== this.reqId) return false;
        const items = stampCullTags(result.items);
        this.publish(initial ? items : [...this.assets, ...items]);
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

  async jumpTo(assetId: string, options: JumpOptions = {}): Promise<boolean> {
    return (await this.openWindow(assetId, options)) === 'accepted';
  }

  async resume(assetId: string | null): Promise<boolean> {
    if (assetId && this.metadataBacked) {
      const outcome = await this.openWindow(assetId, { failure: null });
      if (outcome !== 'failed') return outcome === 'accepted';
    }
    return this.fetchPage(true);
  }

  private async openWindow(assetId: string, options: JumpOptions): Promise<JumpOutcome> {
    if (!this.metadataBacked) return 'failed';
    const jump = (this.#jump += 1);
    const req = this.reqId;
    const current = () => jump === this.#jump && req === this.reqId;
    const failure = options.failure === undefined ? 'Jump to photo' : options.failure;
    const filter = this.currentFilter();
    const fetchWindowed = this.opts.windowFetcher ?? fetchWindow;
    if (!this.loadedOnce) this.loading = true;
    try {
      const [result] = await Promise.all([
        fetchWindowed(assetId, filter, options.reach ?? JUMP_REACH),
        loadCullTags().catch(() => undefined)
      ]);
      if (!current()) return 'stale';
      if (!Array.isArray(result.items) || result.items.length === 0) {
        if (failure) toasts.push('error', `${failure}: the photo is not in this view`);
        return 'failed';
      }
      if (this.collapsed(result)) return (await this.fetchPage(true)) ? 'accepted' : 'failed';
      this.acceptWindow(result);
      return 'accepted';
    } catch (e) {
      if (!current()) return 'stale';
      if (failure) toasts.fail(failure, e);
      return 'failed';
    } finally {
      if (current() && !this.loadedOnce) this.loading = false;
    }
  }

  private collapsed(result: SearchWindow): boolean {
    return (
      singleAnchor(result.items) &&
      result.hasBefore !== true &&
      result.hasAfter !== true &&
      this.totalCount !== 1
    );
  }

  private acceptWindow(result: SearchWindow): void {
    this.reqId += 1;
    this.leaveWindow();
    if (!this.loadingAll) this.loadingMore = false;
    if (this.serverKey() !== this.#countKey) this.startCount(this.currentFilter());
    this.nextCursor = null;
    this.windowed = true;
    this.#before = result.hasBefore === true;
    this.#after = result.hasAfter === true;
    this.publish(stampCullTags(result.items));
    this.loading = false;
    this.loadedOnce = true;
  }

  private setOpen(side: WindowSide, open: boolean): void {
    if (side === 'before') this.#before = open;
    else this.#after = open;
  }

  private async extendWindow(side: WindowSide): Promise<boolean> {
    const req = this.reqId;
    const end: WindowEnd = side === 'before' ? 'first' : 'last';
    const edges = windowEdges(this.assets, end, EDGE_RETRIES + 1);
    const edge = edges[0];
    if (!edge) return false;
    const reach =
      side === 'before' ? { before: EXTEND_REACH, after: 0 } : { before: 0, after: EXTEND_REACH };
    const fetchWindowed = this.opts.windowFetcher ?? fetchWindow;
    const current = () => req === this.reqId && windowEdge(this.assets, end) === edge;
    try {
      for (const anchor of edges) {
        const [result] = await Promise.all([
          fetchWindowed(anchor, this.currentFilter(), reach),
          loadCullTags().catch(() => undefined)
        ]);
        if (!current()) return false;
        if (!Array.isArray(result.items) || result.items.length === 0) continue;
        this.setOpen(
          side,
          side === 'before' ? result.hasBefore === true : result.hasAfter === true
        );
        this.publish(mergeWindow(this.assets, stampCullTags(result.items), side));
        return true;
      }
    } catch (e) {
      if (current()) toasts.fail('load', e);
      return false;
    }
    if (!current()) return false;
    this.setOpen(side, false);
    toasts.push(
      'error',
      side === 'before' ? 'Could not load earlier photos' : 'Could not load more photos'
    );
    return false;
  }

  loadMore(): Promise<boolean> {
    if (this.pendingMore) return this.pendingMore;
    if (this.loadingMore || !this.hasMore) return Promise.resolve(false);
    this.loadingMore = true;
    const load = this.windowed ? this.extendWindow('after') : this.fetchPage(false);
    const pending = load.finally(() => {
      if (this.pendingMore !== pending) return;
      this.pendingMore = null;
      if (!this.loadingAll) this.loadingMore = false;
    });
    this.pendingMore = pending;
    return pending;
  }

  loadPrevious(): Promise<boolean> {
    if (this.pendingPrevious) return this.pendingPrevious;
    if (!this.hasPrevious) return Promise.resolve(false);
    this.loadingPrevious = true;
    const pending = this.extendWindow('before').finally(() => {
      if (this.pendingPrevious !== pending) return;
      this.pendingPrevious = null;
      this.loadingPrevious = false;
    });
    this.pendingPrevious = pending;
    return pending;
  }

  async loadEdge(end: WindowEnd): Promise<boolean> {
    if (end === 'first') return !this.hasPrevious || this.fetchPage(true);
    if (!this.hasMore) return true;
    if (!this.metadataBacked) return false;
    let last: string | null;
    try {
      last = await searchFirst(this.currentFilter(), reversed(browseControls.sort));
    } catch (e) {
      toasts.fail('Jump to the end', e);
      return false;
    }
    return last !== null && this.jumpTo(last, { reach: EDGE_REACH, failure: 'Jump to the end' });
  }

  async loadAll(signal?: AbortSignal): Promise<boolean> {
    if (this.loadingMore || this.loadingAll) return false;
    this.loadingAll = true;
    this.loadingMore = true;
    try {
      while (this.hasPrevious) {
        if (signal?.aborted || !(await this.loadPrevious())) return false;
      }
      while (this.hasMore) {
        if (signal?.aborted) return false;
        const loaded = await (this.windowed ? this.extendWindow('after') : this.fetchPage(false));
        if (!loaded) return false;
      }
      return !signal?.aborted;
    } finally {
      this.loadingAll = false;
      this.loadingMore = false;
    }
  }

  watchFilterChange(): void {
    const key = this.serverKey();
    if (this.prevKey && key !== this.prevKey) {
      selection.clear();
      void this.fetchPage(true);
    }
    this.prevKey = key;
  }
}
