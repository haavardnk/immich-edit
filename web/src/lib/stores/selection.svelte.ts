import type { JobTarget } from '$lib/api/jobs';
import type { SearchQuery } from '$lib/types/search';

class SelectionStore {
  selected = $state(new Set<string>());
  anchorId: string | null = null;
  rangeBase = new Set<string>();
  filterQuery = $state<SearchQuery | null>(null);
  filterCount = $state(0);

  get count(): number {
    return this.selected.size;
  }

  get allFiltered(): boolean {
    return this.filterQuery !== null;
  }

  get targetCount(): number {
    return this.filterQuery !== null ? this.filterCount : this.selected.size;
  }

  get active(): boolean {
    return this.selected.size > 0 || this.filterQuery !== null;
  }

  buildTarget(): JobTarget {
    return this.filterQuery !== null
      ? { search: this.filterQuery }
      : { assetIds: [...this.selected] };
  }

  has = (id: string): boolean => this.filterQuery !== null || this.selected.has(id);

  toggle = (id: string): void => {
    this.filterQuery = null;
    const next = new Set(this.selected);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    this.selected = next;
    this.anchorId = id;
    this.rangeBase = new Set(next);
  };

  rangeTarget = (orderedIds: string[], toId: string): Set<string> => {
    if (!this.anchorId) {
      const next = new Set(this.selected);
      if (next.has(toId)) next.delete(toId);
      else next.add(toId);
      return next;
    }
    const anchorIndex = orderedIds.indexOf(this.anchorId);
    const targetIndex = orderedIds.indexOf(toId);
    if (anchorIndex < 0 || targetIndex < 0) {
      const next = new Set(this.selected);
      if (next.has(toId)) next.delete(toId);
      else next.add(toId);
      return next;
    }
    const start = Math.min(anchorIndex, targetIndex);
    const end = Math.max(anchorIndex, targetIndex);
    const next = new Set(this.rangeBase);
    for (const id of orderedIds.slice(start, end + 1)) next.add(id);
    return next;
  };

  range = (orderedIds: string[], toId: string): void => {
    this.filterQuery = null;
    if (!this.anchorId) {
      this.toggle(toId);
      return;
    }
    const a = orderedIds.indexOf(this.anchorId);
    const b = orderedIds.indexOf(toId);
    if (a < 0 || b < 0) {
      this.toggle(toId);
      return;
    }
    this.selected = this.rangeTarget(orderedIds, toId);
  };

  selectLoaded = (ids: string[]): void => {
    this.filterQuery = null;
    this.selected = new Set(ids);
    this.anchorId = ids.at(-1) ?? null;
    this.rangeBase = new Set(ids);
  };

  selectFiltered = (query: SearchQuery, count: number): void => {
    this.filterQuery = query;
    this.filterCount = count;
    this.selected = new Set();
    this.anchorId = null;
    this.rangeBase = new Set();
  };

  clear = (): void => {
    this.selected = new Set();
    this.anchorId = null;
    this.rangeBase = new Set();
    this.filterQuery = null;
    this.filterCount = 0;
  };
}

export const selection = new SelectionStore();
