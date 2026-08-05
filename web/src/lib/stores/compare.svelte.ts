export type CompareMode = 'single' | 'compare' | 'survey';

export interface PaneView {
  zoomed: boolean;
  cx: number;
  cy: number;
}

export const CENTERED: PaneView = { zoomed: false, cx: 0.5, cy: 0.5 };

class CompareStore {
  mode = $state<CompareMode>('single');
  members = $state<string[]>([]);
  focusIndex = $state(0);
  syncView = $state(true);
  views = $state<Record<string, PaneView>>({});

  get focusedId(): string | null {
    return this.members[this.focusIndex] ?? null;
  }

  enter(mode: CompareMode, ids: string[], focus = 0): void {
    this.mode = mode;
    this.members = ids;
    this.focusIndex = Math.min(Math.max(0, focus), Math.max(0, ids.length - 1));
    this.views = {};
  }

  exit(): void {
    this.mode = 'single';
    this.members = [];
    this.focusIndex = 0;
    this.views = {};
  }

  focusDelta(delta: number): void {
    const count = this.members.length;
    if (count === 0) return;
    this.focusIndex = (((this.focusIndex + delta) % count) + count) % count;
  }

  setMember(index: number, id: string): void {
    if (index < 0 || index >= this.members.length) return;
    const previous = this.members[index];
    this.members[index] = id;
    if (previous !== id) delete this.views[previous];
  }

  drop(index: number): void {
    if (index < 0 || index >= this.members.length) return;
    const [removed] = this.members.splice(index, 1);
    delete this.views[removed];
    this.focusIndex = Math.min(this.focusIndex, Math.max(0, this.members.length - 1));
  }

  promote(index: number): void {
    if (index <= 0 || index >= this.members.length) return;
    const [moved] = this.members.splice(index, 1);
    this.members.unshift(moved);
    this.focusIndex = 0;
  }

  keepOnly(index: number): void {
    const id = this.members[index];
    if (!id) return;
    this.members = [id];
    this.focusIndex = 0;
  }

  viewOf(id: string): PaneView {
    return this.views[id] ?? CENTERED;
  }

  applyView(id: string, view: PaneView): void {
    if (this.syncView && this.mode !== 'single') {
      for (const member of this.members) this.views[member] = { ...view };
      return;
    }
    this.views[id] = view;
  }

  resetViews(): void {
    this.views = {};
  }
}

export const compare = new CompareStore();
