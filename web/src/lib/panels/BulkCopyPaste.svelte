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
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore, mdiContentCopy, mdiContentPaste } from '@mdi/js';

  let busy = $state(false);

  let copyId = $derived(
    selection.filterQuery === null && selection.selected.size === 1
      ? [...selection.selected][0]
      : null,
  );

  let canCopy = $derived(copyId !== null && editedThumbs.getHash(copyId) !== undefined);

  async function copy(): Promise<void> {
    if (busy || !canCopy || !copyId) return;
    busy = true;
    try {
      const record = await getEdits(copyId);
      copyDialog.show(manifestToEdits(record.manifest), () =>
        toasts.push('success', 'Copied settings', 3000),
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
        error: 'Failed to queue paste',
      },
    );
    busy = false;
  }

  async function reset(): Promise<void> {
    if (busy || selection.targetCount === 0) return;
    busy = true;
    await runBulkJob((target) => createResetEditsJob(target), {
      success: (count) => `Queued reset on ${count} asset${count === 1 ? '' : 's'}`,
      error: 'Failed to queue reset',
    });
    busy = false;
  }
</script>

<div class="px-4 py-2.5 flex items-center gap-2">
  <button
    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={busy || selection.targetCount === 0}
    onclick={() => void reset()}
    title="Reset edits on selected assets to original"
  >
    <Icon path={mdiRestore} size={14} />
    Reset
  </button>
  <button
    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={busy || !canCopy}
    onclick={() => void copy()}
    title={canCopy
      ? 'Copy edits from selected asset'
      : copyId !== null
        ? 'Selected asset has no edits'
        : 'Select exactly one asset to copy'}
  >
    <Icon path={mdiContentCopy} size={14} />
    Copy
  </button>
  <button
    class="flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 text-xs transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
    disabled={busy || !clipboard.has || selection.targetCount === 0}
    onclick={() => void paste()}
    title={clipboard.has ? 'Paste edits to selected assets' : 'Nothing copied'}
  >
    <Icon path={mdiContentPaste} size={14} />
    Paste
  </button>
</div>
