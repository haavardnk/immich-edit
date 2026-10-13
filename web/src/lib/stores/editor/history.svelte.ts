import type { Edits } from '$lib/types/edits';

const MAX_HISTORY = 50;

function sameValue(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (typeof a !== 'object' || typeof b !== 'object' || a === null || b === null) return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  const left = a as Record<string, unknown>;
  const right = b as Record<string, unknown>;
  const keys = Object.keys(left);
  return (
    keys.length === Object.keys(right).length &&
    keys.every((key) => key in right && sameValue(left[key], right[key]))
  );
}

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
    if (this.cursor >= 0 && sameValue(this.entries[this.cursor], edits)) return;
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
