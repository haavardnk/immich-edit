import type { AssetSummary } from '#lib/types/album.ts';

export type FeedSnapshot = {
  assets: AssetSummary[];
  nextCursor: string | null;
  totalCount: number | undefined;
  windowed: boolean;
  before: boolean;
  after: boolean;
};

export const FEED_CACHE_ENTRIES = 4;
export const FEED_CACHE_MAX_ITEMS = 4000;
export const FEED_CACHE_MAX_AGE_MS = 5 * 60_000;

type Entry = { snapshot: FeedSnapshot; at: number };

export class FeedCache {
  readonly #entries = new Map<string, Entry>();
  readonly #now: () => number;

  constructor(now: () => number = Date.now) {
    this.#now = now;
  }

  put(key: string, snapshot: FeedSnapshot): void {
    this.#entries.delete(key);
    if (snapshot.assets.length > FEED_CACHE_MAX_ITEMS) return;
    this.#entries.set(key, { snapshot, at: this.#now() });
    for (const oldest of this.#entries.keys()) {
      if (this.#entries.size <= FEED_CACHE_ENTRIES) break;
      this.#entries.delete(oldest);
    }
  }

  take(key: string): FeedSnapshot | null {
    const entry = this.#entries.get(key);
    this.#entries.delete(key);
    if (!entry || this.#now() - entry.at > FEED_CACHE_MAX_AGE_MS) return null;
    return entry.snapshot;
  }

  clear(): void {
    this.#entries.clear();
  }
}

export const feedCache = new FeedCache();
