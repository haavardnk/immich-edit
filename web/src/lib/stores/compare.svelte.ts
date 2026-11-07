import { FIT_VIEW, type View } from '#lib/utils/imageViewport.ts';

export type CompareMode = 'single' | 'compare' | 'survey';

export const MAX_PANES = 9;

export const COMPACT_PANES = 4;

const ANY_LAYOUT = (): number => MAX_PANES;

class CompareStore {
  mode = $state<CompareMode>('single');
  members = $state<string[]>([]);
  focusIndex = $state(0);
  syncView = $state(true);
  views = $state<Record<string, View>>({});
  startCount = $state(0);
  #limit = $state.raw(ANY_LAYOUT);

  get focusedId(): string | null {
    return this.members[this.focusIndex] ?? null;
  }

  get paneLimit(): number {
    return Math.min(MAX_PANES, this.#limit());
  }

  get full(): boolean {
    return this.members.length >= this.paneLimit;
  }

  limitBy(limit: () => number): () => void {
    this.#limit = limit;
    return () => {
      if (this.#limit === limit) this.#limit = ANY_LAYOUT;
    };
  }

  get pruned(): boolean {
    return this.members.length < this.startCount;
  }

  enter(mode: CompareMode, ids: string[], focus = 0): void {
    const members = mode === 'survey' ? ids.slice(0, this.paneLimit) : ids;
    this.mode = mode;
    this.members = members;
    this.focusIndex = Math.min(Math.max(0, focus), Math.max(0, members.length - 1));
    this.views = {};
    this.startCount = members.length;
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
    if (this.members.includes(id) || this.full) return;
    const view = this.focusedId ? this.viewOf(this.focusedId) : FIT_VIEW;
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

  viewOf(id: string): View {
    return this.views[id] ?? FIT_VIEW;
  }

  applyView(id: string, view: View, solo = false): void {
    if (this.syncView && !solo && this.mode !== 'single') {
      for (const member of this.members) this.views[member] = { ...view };
      return;
    }
    this.views[id] = view;
  }
}

export const compare = new CompareStore();
