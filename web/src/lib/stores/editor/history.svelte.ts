import type { Edits } from '$lib/types/edits';

const MAX_HISTORY = 50;

export interface HistoryCtx {
  edits: Edits;
  onCommit(action?: string): Promise<void>;
}

export class EditHistory {
  private ctx: HistoryCtx;
  private entries = $state<Edits[]>([]);
  private cursor = $state(-1);
  private skip = false;

  constructor(ctx: HistoryCtx) {
    this.ctx = ctx;
  }

  get canUndo(): boolean {
    return this.cursor > 0;
  }

  get canRedo(): boolean {
    return this.cursor < this.entries.length - 1;
  }

  get skipping(): boolean {
    return this.skip;
  }

  push(edits: Edits): void {
    const trimmed = this.entries.slice(0, this.cursor + 1);
    trimmed.push(edits);
    if (trimmed.length > MAX_HISTORY) trimmed.shift();
    this.entries = trimmed;
    this.cursor = this.entries.length - 1;
  }

  reset(): void {
    this.entries = [];
    this.cursor = -1;
  }

  undo(): void {
    if (!this.canUndo) return;
    this.cursor--;
    this.replay();
  }

  redo(): void {
    if (!this.canRedo) return;
    this.cursor++;
    this.replay();
  }

  private replay(): void {
    this.ctx.edits = $state.snapshot(this.entries[this.cursor]) as Edits;
    this.skip = true;
    void this.ctx.onCommit();
    this.skip = false;
  }
}
