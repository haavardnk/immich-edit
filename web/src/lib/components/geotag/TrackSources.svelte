<script lang="ts">
  import FileImportButton from '#lib/components/FileImportButton.svelte';
  import { TRACK_EXTENSIONS } from '#lib/geotag/trackFile.ts';
  import { trackSession } from '#lib/stores/trackSession.svelte.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiUpload } from '@mdi/js';

  const session = trackSession;
</script>

<section class="px-4 py-3" aria-label="Track">
  <div
    class="flex flex-col items-center gap-2 rounded-xl border border-dashed border-light-300 px-4 py-5 text-center"
  >
    <Icon icon={mdiUpload} size="24px" class="text-primary" aria-hidden />
    <p class="text-sm text-light-950">
      Drop a GPX, KML or TCX file on the map, or
      <FileImportButton
        accept={TRACK_EXTENSIONS.join(',')}
        label="Choose track files"
        multiple
        onfile={(file) => session.addFiles([file])}
      >
        {#snippet button({ busy, open })}
          <Button
            size="tiny"
            variant="ghost"
            class="px-1 align-baseline"
            loading={busy || session.loading}
            onclick={open}
          >
            choose files
          </Button>
        {/snippet}
      </FileImportButton>
    </p>
    <p class="text-caption text-dark/65">
      From a watch, phone app or GPS logger. Files stay in this browser; only the locations you
      place are saved to Immich.
    </p>
  </div>
</section>
