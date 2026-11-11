import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';
import { sameValue } from '#lib/utils/sameValue.ts';

const MAX_HISTORY = 50;

export interface RestoreMarker {
  from: number;
  to: number;
}

interface HistoryStep {
  id: number;
  edits: Edits;
  restore?: RestoreMarker;
}

export interface HistoryCtx {
  edits: Edits;
  onCommit(action?: string): Promise<void>;
  onLive(): void;
}

export interface EntryRestore {
  id: number;
  edits: Edits;
  from: number | null;
  label: string;
}

export interface HistoryRestores {
  restoreEntry(entryId: number): Promise<boolean>;
  restoreSnapshot(snapshotId: string): Promise<boolean>;
}

interface Checkpoint {
  entries: HistoryStep[];
  cursor: number;
  edits: Edits;
}

function unmarked(steps: HistoryStep[]): HistoryStep[] {
  return steps.map(({ id, edits }) => ({ id, edits }));
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

  push(edits: Edits, restore?: RestoreMarker): number | null {
    if (!restore && this.cursor >= 0 && sameValue(this.entries[this.cursor]?.edits, edits)) {
      return null;
    }
    const kept = this.entries.slice(0, this.cursor + 1);
    const trimmed = restore ? kept : unmarked(kept);
    const id = ++this.nextId;
    trimmed.push(restore ? { id, edits, restore } : { id, edits });
    if (trimmed.length > MAX_HISTORY) trimmed.shift();
    this.entries = trimmed;
    this.cursor = this.entries.length - 1;
    return id;
  }

  get topStep(): number | null {
    return this.entries[this.cursor]?.id ?? null;
  }

  replaceStep(id: number, edits: Edits): boolean {
    if (this.entries[this.cursor]?.id !== id) return false;
    this.entries = this.entries.map((step, index) =>
      index === this.cursor ? { ...step, edits } : step
    );
    return true;
  }

  undoStep(id: number): void {
    if (this.entries[this.cursor]?.id === id) this.undo();
  }

  async restoreEntry({ id, edits, from, label }: EntryRestore): Promise<boolean> {
    const before = this.checkpoint();
    this.show(edits);
    const step = this.push(edits, from === null ? undefined : { from, to: id });
    const restored = await this.settle(this.saves.restoreEntry(id), before);
    if (restored) this.offerUndo(label, step);
    return restored;
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
    const marker = this.entries[this.cursor]?.restore;
    if (marker) this.jump(this.cursor - 1, marker.from);
    else this.replay(this.cursor - 1);
  }

  redo(): void {
    if (!this.canRedo) return;
    const marker = this.entries[this.cursor + 1]?.restore;
    if (marker) this.jump(this.cursor + 1, marker.to);
    else this.replay(this.cursor + 1);
  }

  private jump(target: number, entryId: number): void {
    const step = this.entries[target];
    if (!step) return;
    const before = this.checkpoint();
    this.cursor = target;
    this.show(step.edits);
    void this.settle(this.saves.restoreEntry(entryId), before).catch((e: unknown) =>
      toasts.fail('Could not restore the history step', e)
    );
  }

  private checkpoint(): Checkpoint {
    const edits = $state.snapshot(this.ctx.edits) as Edits;
    return { entries: this.entries, cursor: this.cursor, edits };
  }

  private async settle(restore: Promise<boolean>, before: Checkpoint): Promise<boolean> {
    const landed = { entries: this.entries, cursor: this.cursor };
    let restored = false;
    try {
      restored = await restore;
      return restored;
    } finally {
      if (!restored && this.entries === landed.entries && this.cursor === landed.cursor) {
        this.entries = before.entries;
        this.cursor = before.cursor;
        this.show(before.edits);
      }
    }
  }

  private show(edits: Edits): void {
    this.ctx.edits = $state.snapshot(edits) as Edits;
    this.ctx.onLive();
  }

  private replay(target: number): void {
    const step = this.entries[target];
    if (!step) return;
    this.cursor = target;
    this.entries = unmarked(this.entries);
    this.ctx.edits = $state.snapshot(step.edits) as Edits;
    this.skip = true;
    void this.ctx.onCommit();
    this.skip = false;
  }
}
