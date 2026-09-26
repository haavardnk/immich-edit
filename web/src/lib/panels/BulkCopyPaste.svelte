<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { clipboard } from '$lib/stores/clipboard.svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { createResetEditsJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import { copyEditsFrom, pasteEditsTo } from '$lib/browseCopyPaste';
  import { Button } from '@immich/ui';
  import { mdiRestore, mdiContentCopy, mdiContentPaste } from '@mdi/js';

  let busy = $state(false);

  let copyId = $derived(
    selection.selected.size === 1 ? ([...selection.selected][0] ?? null) : null
  );

  let canCopy = $derived(copyId !== null && editedThumbs.getHash(copyId) !== undefined);

  async function copy(): Promise<void> {
    if (busy || !canCopy || !copyId) return;
    busy = true;
    await copyEditsFrom(copyId);
    busy = false;
  }

  async function paste(): Promise<void> {
    if (busy || !clipboard.has) return;
    busy = true;
    await pasteEditsTo([...selection.selected]);
    busy = false;
  }

  async function reset(): Promise<void> {
    if (busy || selection.count === 0) return;
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
    disabled={busy || selection.count === 0}
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
    disabled={busy || !clipboard.has || selection.count === 0}
    onclick={() => void paste()}
  >
    Paste
  </Button>
</div>
