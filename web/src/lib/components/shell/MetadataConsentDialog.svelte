<script lang="ts">
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
</script>

{#if metadataConsent.open}
  <ConfirmDialog
    title="Sync metadata to Immich"
    confirmLabel="Sync to Immich"
    onConfirm={() => metadataConsent.confirm()}
    onCancel={() => metadataConsent.cancel()}
  >
    <p>
      Ratings, favorites, tags, and reject marks are written back to your Immich library so both
      stay in sync.
    </p>
    <p>
      Photo edits stay non-destructive in local sidecars and originals are untouched. Photos only go
      to the Immich trash when you ask for it.
    </p>
  </ConfirmDialog>
{/if}
