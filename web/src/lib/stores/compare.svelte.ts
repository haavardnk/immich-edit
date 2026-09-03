export type CompareMode = 'single' | 'compare' | 'survey';

export interface PaneView {
  zoom: number;
  cx: number;
  cy: number;
}

export const CENTERED: PaneView = { zoom: 100, cx: 0.5, cy: 0.5 };

export const MAX_PANES = 9;

class CompareStore {
  mode = $state<CompareMode>('single');
  members = $state<string[]>([]);
  focusIndex = $state(0);
  syncView = $state(true);
  views = $state<Record<string, PaneView>>({});
  startCount = $state(0);

  get focusedId(): string | null {
    return this.members[this.focusIndex] ?? null;
  }

  get pruned(): boolean {
    return this.members.length < this.startCount;
  }

  enter(mode: CompareMode, ids: string[], focus = 0): void {
    this.mode = mode;
    this.members = ids;
    this.focusIndex = Math.min(Math.max(0, focus), Math.max(0, ids.length - 1));
    this.views = {};
    this.startCount = ids.length;
  }

  exit(): void {
    this.mode = 'single';
    this.members = [];
    this.focusIndex = 0;
    this.views = {};
    this.startCount = 0;
  }

  focusDelta(delta: number): void {
    const count = this.members.length;
    if (count === 0) return;
    this.focusIndex = (((this.focusIndex + delta) % count) + count) % count;
  }

  setMember(index: number, id: string): void {
    const previous = this.members[index];
    if (previous === undefined || previous === id) return;
    this.members[index] = id;
    const inherited = this.views[previous];
    delete this.views[previous];
    if (inherited) this.views[id] = { ...inherited };
  }

  addMember(id: string): void {
    if (this.members.includes(id) || this.members.length >= MAX_PANES) return;
    const view = this.focusedId ? this.viewOf(this.focusedId) : CENTERED;
    this.members.push(id);
    this.views[id] = { ...view };
    this.focusIndex = this.members.length - 1;
    this.startCount = this.members.length;
    if (this.members.length > 2) this.mode = 'survey';
  }

  drop(index: number): void {
    if (index < 0 || index >= this.members.length) return;
    const [removed] = this.members.splice(index, 1);
    if (removed !== undefined) delete this.views[removed];
    this.focusIndex = Math.min(this.focusIndex, Math.max(0, this.members.length - 1));
  }

  promote(index: number): void {
    if (index <= 0 || index >= this.members.length) return;
    const [moved] = this.members.splice(index, 1);
    if (moved !== undefined) this.members.unshift(moved);
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

  applyView(id: string, view: PaneView, solo = false): void {
    if (this.syncView && !solo && this.mode !== 'single') {
      for (const member of this.members) this.views[member] = { ...view };
      return;
    }
    this.views[id] = view;
  }
}

export const compare = new CompareStore();
