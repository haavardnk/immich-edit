class SelectionStore {
  selected = $state(new Set<string>());
  anchorId: string | null = null;
  rangeBase = new Set<string>();

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
    this.selected = new Set(ids);
    this.anchorId = ids.at(-1) ?? null;
    this.rangeBase = new Set(ids);
  };

  clear = (): void => {
    this.selected = new Set();
    this.anchorId = null;
    this.rangeBase = new Set();
  };
}

export const selection = new SelectionStore();
