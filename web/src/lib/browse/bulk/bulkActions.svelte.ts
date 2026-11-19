import { runBulkJob } from '#lib/api/bulkJob.ts';
import {
  createApplyPresetJob,
  createResetEditsJob,
  type ApplyPresetOptions,
  type Job
} from '#lib/api/jobs.ts';
import { changeAlbums } from '#lib/browse/bulk/bulkAlbums.ts';
import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
import * as bulkMeta from '#lib/browse/bulkMeta.ts';
import type { MultiMode } from '#lib/browse/compareEntry.ts';
import { pasteEditsTo, pasteOnlyTo } from '#lib/browse/copyPaste.ts';
import { openMulti } from '#lib/browse/gridActions.ts';
import { recentTags } from '#lib/stores/recentTags.svelte.ts';
import type { LabelColor } from '#lib/stores/labels.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { toasts } from '#lib/stores/toasts.svelte.ts';
import { trash } from '#lib/stores/trash.svelte.ts';
import { virtualCopies } from '#lib/stores/virtualCopies.svelte.ts';
import type { TagRef } from '#lib/types/asset.ts';
import { firstFocusable } from '#lib/utils/focusable.ts';
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
    if (this.#bar) firstFocusable(this.#bar)?.focus();
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

  changeAlbums = (
    add: readonly string[],
    remove: readonly string[],
    viewing: string | null
  ): Promise<boolean> => this.#report((ids) => changeAlbums(ids, add, remove, viewing));

  changeTags = async (add: TagRef[], remove: TagRef[]): Promise<boolean> => {
    for (const tag of add) recentTags.record(tag);
    return this.#report(async (ids) => {
      const added = add.length === 0 || (await bulkMeta.addTags(ids, add));
      const removed = remove.length === 0 || (await bulkMeta.removeTags(ids, remove));
      return added && removed;
    });
  };

  paste = (): Promise<void> => this.run((ids) => pasteEditsTo(ids));

  pasteOnly = (): void => {
    if (!this.metaBusy) pasteOnlyTo([...selection.selected]);
  };

  resetEdits = (): Promise<void> =>
    this.run((ids) =>
      runBulkJob((target) => createResetEditsJob(target), 'Failed to queue reset', ids)
    );

  applyPreset = async (presetId: string, options: ApplyPresetOptions): Promise<Job | null> => {
    let job: Job | null = null;
    await this.run(async (ids) => {
      job = await runBulkJob(
        (target) => createApplyPresetJob(target, presetId, options),
        'Failed to queue preset',
        ids
      );
    });
    return job;
  };

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

  multi = (mode: MultiMode): void => {
    openMulti(mode, bulkScope.items);
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
