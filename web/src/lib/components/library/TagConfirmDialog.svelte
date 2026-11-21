<script lang="ts">
  import { deleteTagText } from '#lib/browse/tagDialogText.ts';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import DialogLines from '#lib/components/DialogLines.svelte';
  import { tagActions } from '#lib/stores/tagActions.svelte.ts';

  const text = $derived.by(() => {
    const request = tagActions.confirming;
    return request ? deleteTagText(request.tag, request.subtags) : null;
  });
</script>

{#if text}
  <ConfirmDialog
    title={text.title}
    confirmLabel={text.confirm}
    color="danger"
    onConfirm={() => void tagActions.confirm()}
    onCancel={tagActions.cancel}
  >
    <DialogLines lines={text.lines} />
  </ConfirmDialog>
{/if}
