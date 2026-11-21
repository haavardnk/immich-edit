<script lang="ts">
  import { deleteAlbumText, leaveAlbumText } from '#lib/browse/albumDialogText.ts';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import DialogLines from '#lib/components/DialogLines.svelte';
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

{#if text}
  <ConfirmDialog
    title={text.title}
    confirmLabel={text.confirm}
    color="danger"
    onConfirm={() => void albumActions.confirm()}
    onCancel={albumActions.cancel}
  >
    <DialogLines lines={text.lines} />
  </ConfirmDialog>
{/if}
