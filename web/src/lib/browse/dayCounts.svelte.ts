import { searchCounts } from '#lib/api/search.ts';
import { monthCounts } from '#lib/stores/monthCounts.svelte.ts';
import { monthDays, ordered, scrubDays, type ScrubDay } from '#lib/browse/scrubDays.ts';
import type { BrowseFilter, MonthCount, SortDir } from '#lib/types/search.ts';

export class DayCounts {
  #days = $state.raw<ReadonlyMap<string, ScrubDay[]>>(new Map());
  #counts: readonly MonthCount[] | null = null;
  #loading: { month: string; counts: readonly MonthCount[] } | null = null;
  #failed = new Set<string>();
  #queued: string | null = null;
  #filter: () => BrowseFilter;

  constructor(filter: () => BrowseFilter) {
    this.#filter = filter;
  }

  get(month: string, direction: SortDir): ScrubDay[] | null {
    if (this.#counts !== monthCounts.months) return null;
    const days = this.#days.get(month);
    return days ? ordered(days, direction) : null;
  }

  want(month: string): void {
    if (this.#counts !== monthCounts.months) {
      this.#counts = monthCounts.months;
      this.#days = new Map();
      this.#failed = new Set();
    }
    if (this.#days.has(month) || this.#failed.has(month)) return;
    const loading = this.#loading;
    if (loading?.month === month && loading.counts === monthCounts.months) return;
    if (loading) {
      this.#queued = month;
      return;
    }
    void this.load(month);
  }

  private async load(month: string): Promise<void> {
    const days = monthDays(month);
    if (!days) return;
    const counts = monthCounts.months;
    this.#loading = { month, counts };
    try {
      const found = await searchCounts(this.#filter(), days.edges);
      if (counts === monthCounts.months) {
        this.#days = new Map(this.#days).set(month, scrubDays(days.keys, found));
      }
    } catch {
      if (counts === monthCounts.months) this.#failed.add(month);
    } finally {
      this.#loading = null;
      const next = this.#queued;
      this.#queued = null;
      if (next) this.want(next);
    }
  }
}
