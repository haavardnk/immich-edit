import { deleteCopy, listCopies, renameCopy, type CopyRecord } from '#lib/api/copies.ts';
import { persistedPreviewUrl } from '#lib/api/preview.ts';
import { sourceId } from '#lib/browse/assetKey.ts';
import { layout } from '#lib/device/layout.svelte.ts';
import { versionsOf, type Version } from '#lib/editor/versions.ts';
import { budgetFor } from '#lib/render/budget.ts';
import { browsing } from '#lib/stores/browsing.svelte.ts';
import { VersionHold } from '#lib/stores/editor/versionHold.svelte.ts';
import { VersionSummaries } from '#lib/stores/editor/versionSummaries.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { ui } from '#lib/stores/ui.svelte.ts';

export interface VersionsCtx {
  readonly assetId: string | null;
  readonly compare: { versionRemoved(id: string): void };
}

interface CopyList {
  master: string;
  copies: readonly CopyRecord[];
}

export class VersionStore {
  private list = $state.raw<CopyList | null>(null);
  private generation = 0;
  private loaded: string | null = null;
  private loading: { id: string; done: Promise<void> } | null = null;
  private ctx: VersionsCtx;
  readonly hold: VersionHold;
  readonly summaries = new VersionSummaries();

  constructor(ctx: VersionsCtx) {
    this.ctx = ctx;
    this.hold = new VersionHold({
      current: () => this.current,
      url: (id) => persistedPreviewUrl(id, budgetFor(layout.mode).liveEdge, ui.clipWarn)
    });
  }

  get master(): string | null {
    const id = this.ctx.assetId;
    return id ? sourceId(id) : null;
  }

  get copies(): readonly CopyRecord[] {
    const list = this.list;
    return list && list.master === this.master ? list.copies : [];
  }

  get items(): Version[] {
    const master = this.master;
    return master ? versionsOf(master, this.copies) : [];
  }

  get current(): Version | null {
    return this.items.find((version) => version.id === this.ctx.assetId) ?? null;
  }

  load(): Promise<void> {
    const id = this.ctx.assetId;
    if (!id || this.loaded === id) return Promise.resolve();
    if (this.loading?.id === id) return this.loading.done;
    const done = this.fetch(id);
    this.loading = { id, done };
    return done;
  }

  async rename(id: string, name: string): Promise<boolean> {
    try {
      const updated = await renameCopy(id, name.trim() || null);
      this.update((copies) => copies.map((copy) => (copy.id === updated.id ? updated : copy)));
      browsing.patch(updated.id, { copyLabel: updated.name ?? undefined });
      return true;
    } catch (e) {
      toasts.fail('Could not rename the copy', e);
      return false;
    }
  }

  async remove(id: string): Promise<boolean> {
    try {
      await deleteCopy(id);
    } catch (e) {
      toasts.fail('Could not delete the copy', e);
      return false;
    }
    this.update((copies) => copies.filter((copy) => copy.id !== id));
    browsing.remove(id);
    this.summaries.forget(id);
    this.hold.end(id);
    this.ctx.compare.versionRemoved(id);
    return true;
  }

  reset(): void {
    this.generation++;
    this.loaded = null;
    this.loading = null;
    this.hold.end();
  }

  private async fetch(id: string): Promise<void> {
    const generation = ++this.generation;
    const copies = await listCopies(id).catch(() => null);
    if (generation !== this.generation) return;
    this.loading = null;
    if (!copies) return;
    this.list = { master: sourceId(id), copies };
    this.loaded = id;
  }

  private update(change: (copies: readonly CopyRecord[]) => readonly CopyRecord[]): void {
    const list = this.list;
    if (list) this.list = { master: list.master, copies: change(list.copies) };
  }
}
