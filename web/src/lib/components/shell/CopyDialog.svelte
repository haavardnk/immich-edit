<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import CopySectionsActions from '#lib/components/editor/CopySectionsActions.svelte';
  import CopySectionsForm from '#lib/components/editor/CopySectionsForm.svelte';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { Button } from '@immich/ui';
  import { hasSelectedSections } from '#lib/editor/copyPaste.ts';

  const canCopy = $derived(hasSelectedSections(copyDialog.sections));
</script>

{#if copyDialog.open}
  <Dialog title="Copy settings" onClose={copyDialog.cancel}>
    {#snippet actions()}
      <CopySectionsActions />
    {/snippet}

    {#snippet footer()}
      <div class="flex w-full justify-end gap-2">
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="bg-light-100 hover:bg-light-200"
          onclick={copyDialog.cancel}
        >
          Cancel
        </Button>
        <Button size="tiny" color="primary" disabled={!canCopy} onclick={copyDialog.confirm}>
          Copy
        </Button>
      </div>
    {/snippet}

    <CopySectionsForm />
  </Dialog>
{/if}
