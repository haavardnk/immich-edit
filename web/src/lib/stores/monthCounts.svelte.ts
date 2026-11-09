import { searchMonths } from '#lib/api/search.ts';
import type { BrowseFilter, MonthCount } from '#lib/types/search.ts';

export const MONTHS_REFRESH_MS = 300;

export class MonthCountsStore {
  months = $state.raw<MonthCount[]>([]);
  loaded = $state(false);
  #key = '';
  #filter: BrowseFilter | null = null;
  #stale = false;
  #watchers = 0;
  #request = 0;
  #timer: ReturnType<typeof setTimeout> | undefined;

  ensure(filter: BrowseFilter): void {
    const key = JSON.stringify(filter);
    if (key === this.#key && !this.#stale) return;
    if (key !== this.#key) {
      this.months = [];
      this.loaded = false;
    }
    this.#key = key;
    this.#filter = filter;
    this.#stale = false;
    clearTimeout(this.#timer);
    const request = (this.#request += 1);
    searchMonths(filter)
      .then((result) => {
        if (request !== this.#request) return;
        this.months = Array.isArray(result.months) ? result.months : [];
        this.loaded = true;
      })
      .catch(() => {
        if (request !== this.#request) return;
        this.#key = '';
      });
  }

  watch(filter: BrowseFilter): () => void {
    this.#watchers += 1;
    this.ensure(filter);
    return () => {
      this.#watchers -= 1;
    };
  }

  invalidate(): void {
    this.#stale = true;
    clearTimeout(this.#timer);
    if (this.#watchers === 0) return;
    this.#timer = setTimeout(() => {
      const filter = this.#filter;
      if (filter && this.#stale && this.#watchers > 0) this.ensure(filter);
    }, MONTHS_REFRESH_MS);
  }
}

export const monthCounts = new MonthCountsStore();
