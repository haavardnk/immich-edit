<script lang="ts">
  import Dialog from '$lib/components/Dialog.svelte';
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';
  import { Button } from '@immich/ui';
</script>

{#if metadataConsent.open}
  <Dialog title="Sync metadata to Immich" onClose={() => metadataConsent.cancel()}>
    {#snippet footer()}
      <div class="flex w-full justify-end gap-2">
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          onclick={() => metadataConsent.cancel()}
        >
          Cancel
        </Button>
        <Button size="tiny" color="primary" onclick={() => metadataConsent.confirm()}>
          Sync to Immich
        </Button>
      </div>
    {/snippet}

    <div class="text-xs text-dark/80 flex flex-col gap-2">
      <p>
        Ratings, favorites, tags, and reject marks are written back to your Immich library so both
        stay in sync.
      </p>
      <p>
        immich-edit never deletes assets, and photo edits stay non-destructive in local sidecars.
        Originals are untouched.
      </p>
    </div>
  </Dialog>
{/if}
