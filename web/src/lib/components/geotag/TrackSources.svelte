<script lang="ts">
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { TRACK_EXTENSIONS } from '#lib/geotag/trackFile.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Icon, LoadingSpinner } from '@immich/ui';
  import { mdiUpload } from '@mdi/js';

  const session = trackSession;
  const droppable = $derived(layout.mode !== 'touch');
</script>

<section class="px-4 py-3" aria-label="Track">
  <FileImportButton
    accept={TRACK_EXTENSIONS.join(',')}
    label="Choose track files"
    multiple
    onfile={(file) => session.addFiles([file])}
  >
    {#snippet button({ busy, open })}
      {@const loading = busy || session.loading}
      <button
        type="button"
        class="flex w-full flex-col items-center gap-1 rounded-xl border border-dashed border-light-300 px-4 py-5 text-center transition-colors hover:border-primary/60 hover:bg-primary/5 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary active:bg-primary/10 disabled:cursor-progress"
        disabled={loading}
        aria-busy={loading}
        onclick={open}
      >
        <span class="mb-1 flex size-6 items-center justify-center text-primary" aria-hidden="true">
          {#if loading}
            <LoadingSpinner size="medium" />
          {:else}
            <Icon icon={mdiUpload} size="24px" />
          {/if}
        </span>
        <span class="text-sm font-medium text-primary">Choose GPX, KML or TCX files</span>
        {#if droppable}
          <span class="text-sm text-dark/80">or drop them on the map</span>
        {/if}
        <span class="mt-1 text-caption text-dark/65">Files stay in your browser.</span>
      </button>
    {/snippet}
  </FileImportButton>
</section>
