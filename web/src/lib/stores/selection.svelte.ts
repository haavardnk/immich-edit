import type { JobTarget } from '$lib/api/jobs';

class SelectionStore {
  selected = $state(new Set<string>());
  anchorId: string | null = null;
  rangeBase = new Set<string>();
  filterQuery = $state<Record<string, unknown> | null>(null);
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
    const lo = Math.min(a, b);
    const hi = Math.max(a, b);
    const next = new Set(this.rangeBase);
    for (const id of orderedIds.slice(lo, hi + 1)) {
      next.add(id);
    }
    this.selected = next;
  };

  selectLoaded = (ids: string[]): void => {
    this.filterQuery = null;
    this.selected = new Set(ids);
    this.anchorId = ids.at(-1) ?? null;
    this.rangeBase = new Set(ids);
  };

  selectFiltered = (query: Record<string, unknown>, count: number): void => {
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
