<script lang="ts">
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import { signOutFlow } from '#lib/auth/signOutFlow.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
</script>

{#if signOutFlow.confirming}
  <ConfirmDialog
    title="Sign out?"
    confirmLabel="Sign out"
    color="danger"
    onConfirm={() => void signOutFlow.confirm()}
    onCancel={signOutFlow.cancel}
  >
    <p>{countOf(jobs.activeTransfers, 'export or upload job')} still running. Sign out anyway?</p>
  </ConfirmDialog>
{/if}
