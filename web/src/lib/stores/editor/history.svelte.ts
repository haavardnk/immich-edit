import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';

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

interface HistoryStep {
  id: number;
  edits: Edits;
}

export interface HistoryCtx {
  edits: Edits;
  onCommit(action?: string): Promise<void>;
}

export interface HistoryRestores {
  restoreSnapshot(snapshotId: string): Promise<boolean>;
}

export class EditHistory {
  private ctx: HistoryCtx;
  private saves: HistoryRestores;
  private entries = $state<HistoryStep[]>([]);
  private cursor = $state(-1);
  private skip = false;
  private nextId = 0;

  constructor(ctx: HistoryCtx, saves: HistoryRestores) {
    this.ctx = ctx;
    this.saves = saves;
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

  push(edits: Edits): number | null {
    if (this.cursor >= 0 && sameValue(this.entries[this.cursor]?.edits, edits)) return null;
    const trimmed = this.entries.slice(0, this.cursor + 1);
    const id = ++this.nextId;
    trimmed.push({ id, edits });
    if (trimmed.length > MAX_HISTORY) trimmed.shift();
    this.entries = trimmed;
    this.cursor = this.entries.length - 1;
    return id;
  }

  undoStep(id: number): void {
    if (this.entries[this.cursor]?.id === id) this.undo();
  }

  async restoreSnapshot(snapshotId: string, name: string): Promise<boolean> {
    const restored = await this.saves.restoreSnapshot(snapshotId);
    if (restored) this.offerUndo(name, this.push($state.snapshot(this.ctx.edits) as Edits));
    return restored;
  }

  private offerUndo(name: string, step: number | null): void {
    toasts.push('success', `Restored ${name}`, {
      action: step === null ? undefined : { label: 'Undo', run: () => this.undoStep(step) }
    });
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
    const step = this.entries[this.cursor];
    if (!step) return;
    this.ctx.edits = $state.snapshot(step.edits) as Edits;
    this.skip = true;
    void this.ctx.onCommit();
    this.skip = false;
  }
}
