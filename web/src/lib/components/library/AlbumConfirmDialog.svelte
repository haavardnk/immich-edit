<script lang="ts">
  import {
    deleteAlbumText,
    leaveAlbumText,
    type AlbumTextPart
  } from '#lib/browse/albumDialogText.ts';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import { albumActions } from '#lib/stores/albumActions.svelte.ts';
  import { session } from '#lib/stores/session.svelte.ts';

  const text = $derived.by(() => {
    const request = albumActions.confirming;
    if (!request) return null;
    return request.kind === 'delete'
      ? deleteAlbumText(request.album, session.user?.id ?? null)
      : leaveAlbumText(request.album);
  });
</script>

{#snippet part(value: AlbumTextPart)}
  {#if typeof value === 'string'}{value}{:else}<b class="font-semibold text-light-950">
      {value.strong}
    </b>{/if}
{/snippet}

{#if text}
  <ConfirmDialog
    title={text.title}
    confirmLabel={text.confirm}
    color="danger"
    onConfirm={() => void albumActions.confirm()}
    onCancel={albumActions.cancel}
  >
    {#each text.lines as line, index (index)}
      <p>
        {#each line as value, at (at)}{@render part(value)}{/each}
      </p>
    {/each}
  </ConfirmDialog>
{/if}
