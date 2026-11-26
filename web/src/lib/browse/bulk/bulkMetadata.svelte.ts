import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
import type { BulkMenuAction } from '#lib/browse/bulk/bulkMenu.ts';
import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
import { captureEditor } from '#lib/stores/captureEditor.svelte.ts';
import { descriptionEditor } from '#lib/stores/descriptionEditor.svelte.ts';
import { selection } from '#lib/stores/selection.svelte.ts';
import { mdiClockEditOutline, mdiTextBoxEditOutline } from '@mdi/js';

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
}

export const bulkMetadata = new BulkMetadata();
