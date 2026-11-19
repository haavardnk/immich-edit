<script lang="ts">
  import { selection } from '#lib/stores/selection.svelte.ts';
  import { clipboard, NOTHING_COPIED } from '#lib/stores/clipboard.svelte.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { bulkActions } from '#lib/browse/bulk/bulkActions.svelte.ts';
  import { copyEditsFrom } from '#lib/browse/copyPaste.ts';
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
</script>

<div class="flex items-center gap-1 px-3 py-1.5">
  <Button
    size="tiny"
    variant="ghost"
    color="secondary"
    class="h-6 flex-1 panel-action"
    leadingIcon={mdiRestore}
    title="Reset edits on selected assets to original"
    disabled={bulkActions.metaBusy || selection.count === 0}
    onclick={() => void bulkActions.resetEdits()}
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
    title={clipboard.has ? 'Paste edits to selected assets' : NOTHING_COPIED}
    disabled={bulkActions.metaBusy || !clipboard.has || selection.count === 0}
    onclick={() => void bulkActions.paste()}
  >
    Paste
  </Button>
</div>
