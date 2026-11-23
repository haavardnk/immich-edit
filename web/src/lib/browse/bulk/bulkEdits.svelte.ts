import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
import type { BulkMenuAction, BulkMenuRow } from '#lib/browse/bulk/bulkMenu.ts';
import { fileStem } from '#lib/browse/bulk/bulkRules.ts';
import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
import { copyEditsFrom } from '#lib/browse/copyPaste.ts';
import { clipboard, NO_EDITS_TO_COPY, NOTHING_COPIED } from '#lib/stores/clipboard.svelte.ts';
import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import {
  mdiClipboardCheckOutline,
  mdiContentCopy,
  mdiContentPaste,
  mdiRestore,
  mdiViewGridPlusOutline
} from '@mdi/js';

export const SELECT_ONE_PHOTO = 'Select one photo';

function copyReason(id: string | null): string | null {
  if (id === null) return SELECT_ONE_PHOTO;
  return editedThumbs.getHash(id) === undefined ? NO_EDITS_TO_COPY : null;
}

class BulkEdits {
  readonly #blocked = $derived(bulkScope.count === 0 || bulkActions.metaBusy);
  readonly #source = $derived(clipboard.source ? `from ${fileStem(clipboard.source)}` : null);
  readonly #copyId = $derived(bulkScope.single?.id ?? null);
  readonly #copyReason = $derived(copyReason(this.#copyId));
  readonly #edited = $derived([...selection.selected].some((id) => editedThumbs.has(id)));

  readonly applyPreset: BulkMenuRow = $derived({
    label: 'Apply preset',
    icon: mdiViewGridPlusOutline,
    bind: null,
    detail: null,
    disabled: this.#blocked
  });

  readonly paste: BulkMenuAction = $derived({
    label: 'Paste edits',
    icon: mdiContentPaste,
    bind: 'pasteEdits',
    detail: clipboard.has ? this.#source : NOTHING_COPIED,
    disabled: this.#blocked || !clipboard.has,
    run: () => void bulkActions.paste()
  });

  readonly pasteOnly: BulkMenuAction = $derived({
    label: 'Paste only…',
    icon: mdiClipboardCheckOutline,
    bind: 'pasteOnly',
    detail: clipboard.has ? null : NOTHING_COPIED,
    disabled: this.#blocked || !clipboard.has,
    run: bulkActions.pasteOnly
  });

  readonly copy: BulkMenuAction = $derived({
    label: 'Copy edits',
    icon: mdiContentCopy,
    bind: 'copyEdits',
    detail: this.#copyReason,
    disabled: bulkActions.metaBusy || this.#copyReason !== null,
    run: () => {
      if (this.#copyId) void copyEditsFrom(this.#copyId);
    }
  });

  readonly reset: BulkMenuAction = $derived({
    label: 'Reset edits',
    icon: mdiRestore,
    bind: null,
    detail: null,
    disabled: this.#blocked || !this.#edited,
    run: () => void bulkActions.resetEdits()
  });
}

export const bulkEdits = new BulkEdits();
