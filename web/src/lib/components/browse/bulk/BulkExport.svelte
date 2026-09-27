<script lang="ts">
  import { selection } from '$lib/stores/selection.svelte';
  import { browsing } from '$lib/stores/browsing.svelte';
  import { EXTENSION_BY_FORMAT } from '$lib/api/export';
  import { createImmichExportJob, createZipExportJob } from '$lib/api/jobs';
  import { runBulkJob } from '$lib/api/bulkJob';
  import { captureDate } from '$lib/filenameTemplate';
  import { Button } from '@immich/ui';
  import { mdiCloudUpload, mdiFolderZip } from '@mdi/js';
  import DestinationToggle from '$lib/panels/export/DestinationToggle.svelte';
  import ExportPresetMenu from '$lib/panels/export/ExportPresetMenu.svelte';
  import ExportSection from '$lib/panels/export/ExportSection.svelte';
  import FilenameTemplateField from '$lib/panels/export/FilenameTemplateField.svelte';
  import FormatOptions from '$lib/panels/export/FormatOptions.svelte';
  import ResizeOptions from '$lib/panels/export/ResizeOptions.svelte';
  import SharpenOptions from '$lib/panels/export/SharpenOptions.svelte';
  import WatermarkOptions from '$lib/panels/export/WatermarkOptions.svelte';
  import ImmichOptions from '$lib/panels/export/ImmichOptions.svelte';
  import { exportSettings } from '$lib/panels/export/exportSettings.svelte';
  import {
    baseOptions,
    ensureLibraryLoaded,
    formatLabel,
    formInvalid,
    immichOptions
  } from '$lib/panels/export/settings';

  const form = $derived(exportSettings.form);
  const destination = $derived(exportSettings.destination);
  let busy = $state(false);

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded(exportSettings.form);
  });

  async function submit(): Promise<void> {
    if (busy) return;
    busy = true;
    await runBulkJob(
      (assetIds) =>
        destination === 'immich'
          ? createImmichExportJob(assetIds, immichOptions(form))
          : createZipExportJob(assetIds, baseOptions(form)),
      'Failed to queue export'
    );
    busy = false;
  }

  let label = $derived(formatLabel(form.format));
  let first = $derived(browsing.assets.find((asset) => selection.selected.has(asset.id)) ?? null);
  let nameExample = $derived(
    first
      ? {
          original: first.originalFileName,
          date: captureDate(first),
          position: 1,
          total: selection.count
        }
      : null
  );
  let invalid = $derived(formInvalid(form));
</script>

<div class="flex flex-col gap-1 px-3 py-1.5">
  <div class="text-[11px] text-dark/65 select-none">
    {selection.count} asset{selection.count === 1 ? '' : 's'} selected
  </div>

  <ExportPresetMenu />
  <DestinationToggle bind:value={exportSettings.destination} downloadLabel="Download ZIP" />

  <ExportSection title="File">
    <FormatOptions bind:form={exportSettings.form} />
  </ExportSection>

  <ExportSection title="Size">
    <ResizeOptions bind:form={exportSettings.form} />
    <SharpenOptions bind:form={exportSettings.form} />
  </ExportSection>

  <ExportSection title="Watermark">
    <WatermarkOptions bind:form={exportSettings.form} />
  </ExportSection>

  <ExportSection title="Name">
    <FilenameTemplateField
      bind:value={exportSettings.form.filenameTemplate}
      example={nameExample}
      extension={EXTENSION_BY_FORMAT[form.format]}
    />
  </ExportSection>

  {#if destination === 'immich'}
    <ExportSection title="Immich">
      <ImmichOptions bind:form={exportSettings.form} />
    </ExportSection>
  {/if}

  <Button
    size="small"
    color="primary"
    fullWidth
    loading={busy}
    leadingIcon={destination === 'immich' ? mdiCloudUpload : mdiFolderZip}
    disabled={busy || selection.count === 0 || invalid}
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
