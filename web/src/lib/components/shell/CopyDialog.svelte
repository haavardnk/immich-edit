<script lang="ts">
  import Dialog from '#lib/components/Dialog.svelte';
  import CopySectionsActions from '#lib/components/editor/CopySectionsActions.svelte';
  import CopySectionsForm from '#lib/components/editor/CopySectionsForm.svelte';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { copyDialog } from '#lib/stores/copyDialog.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiContentCopy, mdiContentPaste } from '@mdi/js';

  const pasting = $derived(copyDialog.mode === 'paste');
</script>

{#snippet kbd(keys: string)}
  <kbd class="rounded border border-hairline px-1 font-mono text-micro text-dark/65">{keys}</kbd>
{/snippet}

{#if copyDialog.open}
  <Dialog
    title={pasting ? 'Paste settings' : 'Copy settings'}
    subtitle={copyDialog.sourceName ? `From ${copyDialog.sourceName}` : undefined}
    size="large"
    onClose={copyDialog.cancel}
  >
    {#snippet actions()}
      <CopySectionsActions />
    {/snippet}

    {#snippet footer()}
      <div class="flex w-full items-center gap-2">
        {#if !pasting}
          <p class="flex min-w-0 items-center gap-1.5 text-caption text-dark/65 touch:hidden">
            <Icon icon={mdiContentPaste} size="14px" class="shrink-0" aria-hidden="true" />
            {@render kbd(keysFor('pasteEdits'))}
            <span>pastes these ·</span>
            {@render kbd(keysFor('pasteOnly'))}
            <span>Paste only…</span>
          </p>
        {/if}
        <div class="ms-auto flex gap-2 touch:w-full">
          <Button
            size="tiny"
            variant="ghost"
            color="secondary"
            class="bg-light-100 hover:bg-light-200 touch:h-12 touch:flex-1 touch:rounded-full touch:text-base"
            onclick={copyDialog.cancel}
          >
            Cancel
          </Button>
          <Button
            size="tiny"
            color="primary"
            class="touch:h-12 touch:flex-1 touch:rounded-full touch:text-base"
            leadingIcon={pasting ? mdiContentPaste : mdiContentCopy}
            disabled={copyDialog.count === 0}
            onclick={copyDialog.confirm}
          >
            {pasting ? 'Paste' : 'Copy'}
          </Button>
        </div>
      </div>
    {/snippet}

    <CopySectionsForm />
  </Dialog>
{/if}
