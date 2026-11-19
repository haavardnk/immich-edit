<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import CopySectionsActions from '#lib/components/editor/CopySectionsActions.svelte';
  import CopySectionsForm from '#lib/components/editor/CopySectionsForm.svelte';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { hasSelectedSections } from '#lib/editor/copyPaste.ts';
  import { mdiClipboardOutline } from '@mdi/js';

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
    <p
      class="mt-4 hidden gap-2.5 rounded-lg bg-light-100 px-3 py-2.5 text-[13px] leading-snug text-dark/65 touch:flex"
    >
      <Icon
        icon={mdiClipboardOutline}
        size="16px"
        class="mt-0.5 shrink-0 text-primary"
        aria-hidden="true"
      />
      <span>
        Then open another photo and use <strong class="text-dark">More › Paste edits</strong>, or
        select photos in the grid and use <strong class="text-dark">⋯ › Paste edits</strong>. Each
        paste is one history step.
      </span>
    </p>
  </Dialog>
{/if}
