import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
import { captureEditor } from '#lib/stores/captureEditor.svelte.ts';
import { descriptionEditor } from '#lib/stores/descriptionEditor.svelte.ts';
import { metadataClipboard } from '#lib/stores/metadataClipboard.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import {
  mdiClockEditOutline,
  mdiContentCopy,
  mdiContentPaste,
  mdiTextBoxEditOutline
} from '@mdi/js';

class BulkMetadata {
  readonly #blocked = $derived(bulkScope.count === 0 || bulkActions.metaBusy);

  readonly captureTime: BulkMenuAction = $derived({
    label: 'Capture time…',
    icon: mdiClockEditOutline,
    bind: null,
    detail: null,
    disabled: this.#blocked,
    run: () => void captureEditor.openIds([...selection.selected])
  });

  readonly description: BulkMenuAction = $derived({
    label: 'Description…',
    icon: mdiTextBoxEditOutline,
    bind: null,
    detail: null,
    disabled: this.#blocked,
    run: () => void descriptionEditor.openIds([...selection.selected])
  });

  readonly copy: BulkMenuAction | null = $derived.by(() => {
    const single = bulkScope.single;
    if (!single) return null;
    return {
      label: 'Copy metadata',
      icon: mdiContentCopy,
      bind: null,
      detail: null,
      disabled: bulkActions.metaBusy,
      run: () => void metadataClipboard.copy(single)
    };
  });

  readonly paste: BulkMenuAction | null = $derived.by(() => {
    const copied = metadataClipboard.copied;
    if (!copied) return null;
    return {
      label: 'Paste metadata…',
      icon: mdiContentPaste,
      bind: null,
      detail: copied.name,
      disabled: this.#blocked,
      run: () => void metadataClipboard.openPaste([...selection.selected])
    };
  });
}

export const bulkMetadata = new BulkMetadata();
