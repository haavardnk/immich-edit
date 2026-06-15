<script lang="ts">
  import { metadataConsent } from '$lib/stores/metadataConsent.svelte';

  function onBackdropClick(e: MouseEvent): void {
    if (e.currentTarget === e.target) metadataConsent.cancel();
  }
</script>

{#if metadataConsent.open}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm"
    role="presentation"
    onclick={onBackdropClick}
  >
    <div
      class="bg-immich-dark-gray border border-white/10 rounded-lg shadow-xl p-5 min-w-[320px] max-w-105"
      role="dialog"
      aria-modal="true"
      aria-label="Sync metadata to Immich"
    >
      <h2 class="text-sm font-medium text-immich-dark-fg mb-3">Sync metadata to Immich</h2>
      <div class="text-xs text-immich-dark-fg/80 flex flex-col gap-2">
        <p>
          Ratings, favorites, tags, and reject marks are written back to your Immich library so both
          stay in sync.
        </p>
        <p>
          immich-edit never deletes assets, and photo edits stay non-destructive in local sidecars.
          Originals are untouched.
        </p>
      </div>
      <div class="flex justify-end gap-2 mt-4">
        <button
          class="text-xs px-3 py-1.5 rounded text-immich-dark-fg/70 hover:bg-white/10 hover:text-immich-dark-fg transition-colors"
          onclick={() => metadataConsent.cancel()}
        >
          Cancel
        </button>
        <button
          class="text-xs px-3 py-1.5 rounded bg-immich-primary text-black font-medium hover:bg-immich-primary/90 transition-colors"
          onclick={() => metadataConsent.confirm()}
        >
          Sync to Immich
        </button>
      </div>
    </div>
  </div>
{/if}
