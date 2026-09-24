<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import TextInput from '$lib/components/TextInput.svelte';
  import { createImmichExportJob, createZipExportJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import { Button } from '@immich/ui';
  import { mdiCloudUpload, mdiFolderZip } from '@mdi/js';
  import DestinationToggle from './export/DestinationToggle.svelte';
  import FormatOptions from './export/FormatOptions.svelte';
  import ImmichOptions from './export/ImmichOptions.svelte';
  import { exportSettings } from './export/exportSettings.svelte';
  import { baseOptions, ensureLibraryLoaded, formatLabel, immichOptions } from './export/settings';

  const form = $derived(exportSettings.form);
  const destination = $derived(exportSettings.destination);
  let busy = $state(false);

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded(exportSettings.form);
  });

  async function submit(): Promise<void> {
    if (busy) return;
    busy = true;
    const verb = destination === 'immich' ? 'export to Immich' : 'zip download';
    await runBulkJob(
      (assetIds) =>
        destination === 'immich'
          ? createImmichExportJob(assetIds, immichOptions(form))
          : createZipExportJob(assetIds, baseOptions(form), form.filenameSuffix),
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
    {selection.count} asset{selection.count === 1 ? '' : 's'} selected
  </div>

  <DestinationToggle bind:value={exportSettings.destination} downloadLabel="Download ZIP" />

  <FormatOptions bind:form={exportSettings.form} />

  <div class="flex flex-col gap-1 border-t border-hairline pt-1.5">
    <TextInput
      label="Filename suffix"
      compact
      color="neutral"
      class="ring-0 focus-within:ring-1 focus-within:ring-primary"
      bind:value={exportSettings.form.filenameSuffix}
      placeholder="_edit"
    />
    {#if destination === 'immich'}
      <ImmichOptions bind:form={exportSettings.form} />
    {/if}
  </div>

  <Button
    size="small"
    color="primary"
    fullWidth
    loading={busy}
    leadingIcon={destination === 'immich' ? mdiCloudUpload : mdiFolderZip}
    disabled={busy || selection.count === 0}
    onclick={() => void submit()}
  >
    {destination === 'immich'
      ? `Export ${selection.count} to Immich`
      : `Download ${selection.count} as ${label} ZIP`}
  </Button>

  <p class="text-[10px] leading-snug text-dark/65">
    Runs as a background job. Track progress in the Jobs panel.
  </p>
</div>
