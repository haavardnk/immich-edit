<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { createImmichExportJob, createZipExportJob, type JobTarget } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import { Button } from '@immich/ui';
  import { mdiCloudUpload, mdiFolderZip } from '@mdi/js';
  import DestinationToggle from './export/DestinationToggle.svelte';
  import FormatOptions from './export/FormatOptions.svelte';
  import ImmichOptions from './export/ImmichOptions.svelte';
  import {
    baseOptions,
    DEFAULT_FILENAME_SUFFIX,
    defaultExportForm,
    ensureLibraryLoaded,
    formatLabel,
    immichOptions,
    type Destination
  } from './export/settings';

  let destination = $state<Destination>('download');
  let form = $state(defaultExportForm());
  let busy = $state(false);

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded();
  });

  async function submit(): Promise<void> {
    if (busy) return;
    busy = true;
    const verb = destination === 'immich' ? 'export to Immich' : 'zip download';
    await runBulkJob(
      (target: JobTarget) =>
        destination === 'immich'
          ? createImmichExportJob(target, immichOptions(form))
          : createZipExportJob(target, baseOptions(form), DEFAULT_FILENAME_SUFFIX),
      {
        success: (count) => `Queued ${verb} of ${count} asset${count === 1 ? '' : 's'}`,
        error: 'Failed to queue export'
      }
    );
    busy = false;
  }

  let label = $derived(formatLabel(form.format));
</script>

<div class="flex flex-col gap-1 px-3 py-1.5">
  <div class="text-[11px] text-dark/65 select-none">
    {selection.targetCount} asset{selection.targetCount === 1 ? '' : 's'} selected
  </div>

  <DestinationToggle bind:value={destination} downloadLabel="Download ZIP" />

  <FormatOptions bind:form />

  {#if destination === 'immich'}
    <div class="flex flex-col gap-1 border-t border-hairline pt-1.5">
      <TextInput
        label="Filename suffix"
        compact
        color="neutral"
        class="ring-0 focus-within:ring-1 focus-within:ring-primary"
        bind:value={form.filenameSuffix}
        placeholder="_edit"
      />
      <ImmichOptions bind:form />
    </div>
  {/if}

  <Button
    size="small"
    color="primary"
    fullWidth
    loading={busy}
    leadingIcon={destination === 'immich' ? mdiCloudUpload : mdiFolderZip}
    disabled={busy || selection.targetCount === 0}
    onclick={() => void submit()}
  >
    {destination === 'immich'
      ? `Export ${selection.targetCount} to Immich`
      : `Download ${selection.targetCount} as ${label} ZIP`}
  </Button>

  <p class="text-[10px] leading-snug text-dark/65">
    Runs as a background job. Track progress in the Jobs panel.
  </p>
</div>
