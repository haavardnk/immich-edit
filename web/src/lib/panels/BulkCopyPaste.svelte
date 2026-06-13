<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { clipboard } from '$lib/stores/clipboard.svelte';
  import { copyDialog } from '$lib/stores/copyDialog.svelte';
  import { editedThumbs } from '$lib/stores/editedThumbs.svelte';
  import { toasts } from '$lib/stores/toasts.svelte';
  import { getEdits } from '$lib/api/edits';
  import { createPasteEditsJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import { manifestToEdits, editsToManifest } from '$lib/types/edits';
  import CopyPasteButtons from '$lib/components/CopyPasteButtons.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiContentCopy } from '@mdi/js';

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
</script>

<div class="flex flex-col gap-2.5 px-4 pt-3 pb-4 border-b border-white/10 bg-white/1.5">
  <div class="flex items-center gap-1.5">
    <Icon path={mdiContentCopy} size={13} class="opacity-50" />
    <span class="uppercase tracking-wider text-[10px] font-medium text-immich-dark-fg/50">Copy &amp; paste</span>
  </div>
  <div class="flex items-center gap-2">
    <CopyPasteButtons
      canCopy={!busy && canCopy}
      canPaste={!busy && clipboard.has && selection.targetCount > 0}
      copyTitle={canCopy
        ? 'Copy edits from selected asset'
        : copyId !== null
          ? 'Selected asset has no edits'
          : 'Select exactly one asset to copy'}
      pasteTitle={clipboard.has ? 'Paste edits to selected assets' : 'Nothing copied'}
      onCopy={() => void copy()}
      onPaste={() => void paste()}
    />
    <span class="text-[11px] leading-snug text-immich-dark-fg/50">
      {#if clipboard.has}
        Paste copied settings to {selection.targetCount} asset{selection.targetCount === 1 ? '' : 's'}
      {:else}
        Select one asset to copy its edits
      {/if}
    </span>
  </div>
</div>
