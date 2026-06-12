class SelectionStore {
  selected = $state(new Set<string>());
  anchorId: string | null = null;

  get count(): number {
    return this.selected.size;
  }

  get active(): boolean {
    return this.selected.size > 0;
  }

  has = (id: string): boolean => this.selected.has(id);

  toggle = (id: string): void => {
    const next = new Set(this.selected);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    this.selected = next;
    this.anchorId = id;
  };

  range = (orderedIds: string[], toId: string): void => {
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
    const next = new Set(this.selected);
    for (const id of orderedIds.slice(lo, hi + 1)) {
      next.add(id);
    }
    this.selected = next;
  };

  selectLoaded = (ids: string[]): void => {
    this.selected = new Set(ids);
    this.anchorId = ids.at(-1) ?? null;
  };

  clear = (): void => {
    this.selected = new Set();
    this.anchorId = null;
  };
}

export const selection = new SelectionStore();
