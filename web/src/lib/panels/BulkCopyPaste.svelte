<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { clipboard } from '$lib/stores/clipboard.svelte';
  import { copyDialog } from '$lib/stores/copyDialog.svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { getEdits } from '$lib/api/edits';
  import { createPasteEditsJob, createResetEditsJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import { manifestToEdits, editsToManifest } from '$lib/types/edits';
  import { Button } from '@immich/ui';
  import { mdiRestore, mdiContentCopy, mdiContentPaste } from '@mdi/js';

  let busy = $state(false);

  let copyId = $derived(
    selection.filterQuery === null && selection.selected.size === 1
      ? [...selection.selected][0]
      : null
  );

  let canCopy = $derived(copyId !== null && editedThumbs.getHash(copyId) !== undefined);

  async function copy(): Promise<void> {
    if (busy || !canCopy || !copyId) return;
    busy = true;
    try {
      const record = await getEdits(copyId);
      copyDialog.show(manifestToEdits(record.manifest), () =>
        toasts.push('success', 'Copied settings', 3000)
      );
    } catch (e) {
      toasts.push('error', `Failed to copy edits: ${(e as Error).message}`, 6000);
    } finally {
      busy = false;
    }
  }

  async function paste(): Promise<void> {
    if (busy) return;
    const snap = clipboard.snapshot();
    if (!snap) return;
    busy = true;
    await runBulkJob(
      (target) => createPasteEditsJob(target, editsToManifest(snap.edits), snap.sections),
      {
        success: (count) => `Queued paste on ${count} asset${count === 1 ? '' : 's'}`,
        error: 'Failed to queue paste'
      }
    );
    busy = false;
  }

  async function reset(): Promise<void> {
    if (busy || selection.targetCount === 0) return;
    busy = true;
    await runBulkJob((target) => createResetEditsJob(target), {
      success: (count) => `Queued reset on ${count} asset${count === 1 ? '' : 's'}`,
      error: 'Failed to queue reset'
    });
    busy = false;
  }
</script>

<div class="flex items-center gap-1 px-3 py-1.5">
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-6 flex-1 panel-action"
    leadingIcon={mdiRestore}
    title="Reset edits on selected assets to original"
    disabled={busy || selection.targetCount === 0}
    onclick={() => void reset()}
  >
    Reset
  </Button>
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-6 flex-1 panel-action"
    leadingIcon={mdiContentCopy}
    title={canCopy
      ? 'Copy edits from selected asset'
      : copyId !== null
        ? 'Selected asset has no edits'
        : 'Select exactly one asset to copy'}
    disabled={busy || !canCopy}
    onclick={() => void copy()}
  >
    Copy
  </Button>
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-6 flex-1 panel-action"
    leadingIcon={mdiContentPaste}
    title={clipboard.has ? 'Paste edits to selected assets' : 'Nothing copied'}
    disabled={busy || !clipboard.has || selection.targetCount === 0}
    onclick={() => void paste()}
  >
    Paste
  </Button>
</div>
