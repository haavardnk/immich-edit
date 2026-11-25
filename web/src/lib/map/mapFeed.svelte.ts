import { assetSummaries } from '#lib/api/map.ts';
import type { WindowEnd } from '#lib/browse/feedWindow.ts';
import { browsing, type BrowsePager } from '#lib/stores/browsing.svelte.ts';
import type { AssetSummary } from '#lib/types/album.ts';

const MAP_PAGE = 60;

function sameIds(a: readonly string[], b: readonly string[]): boolean {
  return a.length === b.length && a.every((id, index) => id === b[index]);
}

export class MapFeed implements BrowsePager {
  assets = $state<AssetSummary[]>([]);
  error = $state<string | null>(null);
  readonly hasPrevious = false;
  #ids = $state<readonly string[]>([]);
  #next = $state(0);
  #loading = $state(false);
  #claim = false;
  #generation = 0;
  #load: (ids: readonly string[]) => Promise<AssetSummary[]>;

  constructor(load: (ids: readonly string[]) => Promise<AssetSummary[]> = assetSummaries) {
    this.#load = load;
  }

  get total(): number {
    return this.#ids.length;
  }

  get hasMore(): boolean {
    return this.#next < this.#ids.length;
  }

  get loadingMore(): boolean {
    return this.#loading;
  }

  attach(): void {
    if (!browsing.isFeed(this.assets, this)) browsing.set(this.assets, this);
    browsing.total = this.#ids.length;
  }

  show(ids: readonly string[]): Promise<boolean> {
    if (sameIds(ids, this.#ids) && (this.assets.length > 0 || ids.length === 0)) {
      this.attach();
      return Promise.resolve(false);
    }
    this.#generation += 1;
    this.#ids = ids;
    this.#next = 0;
    this.#loading = false;
    this.error = null;
    this.assets = [];
    if (!this.hasMore) {
      this.attach();
      return Promise.resolve(false);
    }
    this.#claim = true;
    return this.loadMore();
  }

  async loadMore(): Promise<boolean> {
    if (!this.hasMore || this.#loading) return false;
    const generation = this.#generation;
    const slice = this.#ids.slice(this.#next, this.#next + MAP_PAGE);
    this.#loading = true;
    try {
      const page = await this.#load(slice);
      if (generation !== this.#generation) return false;
      this.#next += slice.length;
      this.assets.push(...page);
      return true;
    } catch (error) {
      if (generation === this.#generation) this.error = String(error);
      return false;
    } finally {
      if (generation === this.#generation) this.#settle();
    }
  }

  #settle(): void {
    this.#loading = false;
    if (!this.#claim) return;
    this.#claim = false;
    this.attach();
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
    if (!this.#ids.includes(assetId)) return false;
    while (!this.assets.some((asset) => asset.id === assetId)) {
      if (!(await this.loadMore())) return false;
    }
    return true;
  }
}
