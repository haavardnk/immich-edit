import { runBulkJob } from '#lib/api/bulkJob.ts';
import {
  createApplyPresetJob,
  createResetEditsJob,
  type ApplyPresetOptions
} from '#lib/api/jobs.ts';
import { addToAlbums, removeFromAlbum } from '#lib/browse/bulk/bulkAlbums.ts';
import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
import * as bulkMeta from '#lib/browse/bulkMeta.ts';
import type { MultiMode } from '#lib/browse/compareEntry.ts';
import { pasteEditsTo } from '#lib/browse/copyPaste.ts';
import { openMulti } from '#lib/browse/gridActions.ts';
import { MAX_PANES } from '#lib/stores/compare.svelte.ts';
import type { LabelColor } from '#lib/stores/labels.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
import type { AlbumSummary } from '#lib/types/album.ts';
import type { TagRef } from '#lib/types/asset.ts';
import type { Attachment } from 'svelte/attachments';

class BulkActions {
  busy = $state(false);
  #bar: HTMLElement | null = null;

  get metaBusy(): boolean {
    return this.busy || bulkScope.selectingAll || virtualCopies.busy;
  }

  barRef: Attachment<HTMLElement> = (element) => {
    this.#bar = element;
    return () => {
      if (this.#bar === element) this.#bar = null;
    };
  };

  focusBar = (): void => {
    this.#bar?.querySelector<HTMLElement>('button:not([disabled]), a[href]')?.focus();
  };

  run = async (write: (ids: string[]) => Promise<unknown>): Promise<void> => {
    if (this.metaBusy) return;
    this.busy = true;
    try {
      await write([...selection.selected]);
    } finally {
      this.busy = false;
    }
  };

  setFavorite = (value: boolean): Promise<void> =>
    this.run((ids) => bulkMeta.setFavorite(ids, value));

  setRating = (value: number | null): Promise<void> =>
    this.run((ids) => bulkMeta.setRating(ids, value));

  setRejected = (value: boolean): Promise<void> =>
    this.run((ids) => bulkMeta.setRejected(ids, value));

  setLabel = (color: LabelColor | null): Promise<void> =>
    this.run((ids) => bulkMeta.setLabel(ids, color));

  createCopies = (): Promise<void> => this.run(virtualCopies.create);

  setTags = (tags: TagRef[], add: boolean): Promise<boolean> =>
    this.#report((ids) => (add ? bulkMeta.addTags(ids, tags) : bulkMeta.removeTags(ids, tags)));

  addToAlbums = (albumIds: readonly string[]): Promise<boolean> =>
    this.#report((ids) => addToAlbums(ids, albumIds));

  removeFromAlbum = (album: AlbumSummary): Promise<void> =>
    this.run((ids) => removeFromAlbum(ids, album));

  paste = (): Promise<void> => this.run((ids) => pasteEditsTo(ids));

  resetEdits = (): Promise<void> =>
    this.run((ids) =>
      runBulkJob((target) => createResetEditsJob(target), 'Failed to queue reset', ids)
    );

  applyPreset = (presetId: string, options: ApplyPresetOptions): Promise<void> =>
    this.run((ids) =>
      runBulkJob(
        (target) => createApplyPresetJob(target, presetId, options),
        'Failed to queue preset',
        ids
      )
    );

  trash = (): void => {
    const reason = bulkScope.trashReason;
    if (reason) {
      toasts.push('info', reason);
      return;
    }
    void trash.request(bulkScope.picked);
  };

  ownedOrExplain = (): boolean => {
    if (bulkScope.allOwned) return true;
    toasts.push('info', bulkScope.notOwnedReason);
    return false;
  };

  multi = (mode: MultiMode, panes: number = MAX_PANES): void => {
    openMulti(mode, bulkScope.items, panes);
  };

  selectAll = async (): Promise<void> => {
    if (this.busy || bulkScope.loadingMore || bulkScope.selectingAll) return;
    await bulkScope.selectAll();
  };

  clear = (): void => {
    if (bulkScope.selectingAll) bulkScope.cancelSelectAll();
    selection.clear();
  };

  async #report(write: (ids: string[]) => Promise<boolean>): Promise<boolean> {
    let done = false;
    await this.run(async (ids) => {
      done = await write(ids);
    });
    return done;
  }
}

export const bulkActions = new BulkActions();
