<script lang="ts">
  import { trashDialogText } from '#lib/browse/trash.ts';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import { immichCapabilities } from '#lib/stores/immichCapabilities.svelte.ts';
  import { trash } from '#lib/stores/trash.svelte.ts';

  const text = $derived(
    trash.plan ? trashDialogText(trash.plan, immichCapabilities.trashDays) : null
  );
</script>

{#if text}
  <ConfirmDialog
    title={text.title}
    confirmLabel={text.confirm}
    color="danger"
    onConfirm={trash.confirm}
    onCancel={trash.cancel}
  >
    {#each text.lines as line (line)}
      <p>{line}</p>
    {/each}
  </ConfirmDialog>
{/if}
