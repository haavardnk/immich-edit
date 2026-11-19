import { listEditHistory, type EditHistoryEntry } from '#lib/api/edits.ts';
import { currentEntry, entryEdits, entryText, stepNumber } from '#lib/editor/historySteps.ts';
import type { PillText } from '#lib/editor/pill.ts';
import type { EditsPreviewOwner } from '#lib/stores/editor/editsPreview.ts';
import type { EntryRestore } from '#lib/stores/editor/history.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import type { Edits } from '#lib/types/edits.ts';

export interface HistoryStepsCtx {
  readonly assetId: string | null;
  readonly isRaw: boolean;
  readonly savedHash: string;
  readonly restoreCount: number;
  settled(): Promise<void>;
  restoreHistoryEntry(restore: EntryRestore): Promise<boolean>;
  previewEdits(owner: EditsPreviewOwner, edits: Edits): void;
  endEditsPreview(owner: EditsPreviewOwner): void;
}

export class HistorySteps {
  entries = $state.raw<EditHistoryEntry[]>([]);
  loading = $state(true);
  busyId = $state<number | null>(null);
  hoveredId = $state<number | null>(null);
  sheetOpen = $state(false);
  readonly current = $derived(currentEntry(this.entries));
  private generation = 0;
  private loaded: string | null = null;
  private fetching: { key: string; done: Promise<void> } | null = null;
  private ctx: HistoryStepsCtx;

  constructor(ctx: HistoryStepsCtx) {
    this.ctx = ctx;
  }

  get pill(): PillText | null {
    const hovered = this.entries.find((entry) => entry.id === this.hoveredId);
    if (hovered) {
      return {
        lead: 'Previewing',
        label: this.label(hovered),
        tail: '· move off the list to return',
        mono: true
      };
    }
    const current = this.current;
    if (!this.sheetOpen || !current) return null;
    return {
      lead: `Step ${stepNumber(this.entries, current.id)} of ${this.entries.length} ·`,
      label: this.label(current),
      tail: '',
      mono: false
    };
  }

  get revision(): string {
    return `${this.ctx.assetId}:${this.ctx.savedHash}:${this.ctx.restoreCount}`;
  }

  label(entry: EditHistoryEntry): string {
    return entryText(this.entries, entry, this.ctx.isRaw);
  }

  refresh(): Promise<void> {
    const assetId = this.ctx.assetId;
    if (!assetId) {
      this.clear();
      this.loading = false;
      return Promise.resolve();
    }
    const key = this.revision;
    if (this.fetching?.key === key) return this.fetching.done;
    if (this.loaded === key) return Promise.resolve();
    const done = this.fetch(assetId, key);
    this.fetching = { key, done };
    return done;
  }

  async restore(entry: EditHistoryEntry): Promise<void> {
    if (this.busyId !== null || entry.id === this.current?.id) return;
    this.leave();
    this.busyId = entry.id;
    try {
      await this.ctx.settled();
      await this.refresh();
      const from = this.current;
      const target = this.entries.find((item) => item.id === entry.id);
      const edits = target && entryEdits(target);
      if (!target || !edits || target.id === from?.id) return;
      await this.ctx.restoreHistoryEntry({
        id: target.id,
        edits,
        from: from?.id ?? null,
        label: this.label(target)
      });
      await this.refresh();
    } catch (e) {
      toasts.fail('Could not restore the history step', e);
    } finally {
      this.busyId = null;
    }
  }

  hover(entry: EditHistoryEntry): void {
    const edits = entryEdits(entry);
    if (!edits || this.busyId !== null || entry.id === this.current?.id) {
      this.leave();
      return;
    }
    this.hoveredId = entry.id;
    this.ctx.previewEdits('history', edits);
  }

  leave(): void {
    if (this.hoveredId === null) return;
    this.hoveredId = null;
    this.ctx.endEditsPreview('history');
  }

  reset(): void {
    this.clear();
    this.loading = true;
    this.busyId = null;
    this.hoveredId = null;
  }

  private clear(): void {
    this.generation++;
    this.loaded = null;
    this.fetching = null;
    this.entries = [];
  }

  private async fetch(assetId: string, key: string): Promise<void> {
    const generation = ++this.generation;
    this.loading = this.entries.length === 0;
    const entries = await listEditHistory(assetId).catch(() => null);
    if (generation !== this.generation) return;
    this.fetching = null;
    this.loaded = entries ? key : null;
    this.entries = entries ?? [];
    this.loading = false;
  }
}
