<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiExport, mdiCloudUpload, mdiRefresh, mdiAlertOutline } from '@mdi/js';
  import { EXTENSION_BY_FORMAT } from '$lib/api/export';
  import { captureDate } from '$lib/filenameTemplate';
  import { croppedOutputSize } from '$lib/utils/geom';
  import DestinationToggle from './export/DestinationToggle.svelte';
  import ExportSection from './export/ExportSection.svelte';
  import FilenameTemplateField from './export/FilenameTemplateField.svelte';
  import FormatOptions from './export/FormatOptions.svelte';
  import ResizeOptions from './export/ResizeOptions.svelte';
  import SharpenOptions from './export/SharpenOptions.svelte';
  import WatermarkOptions from './export/WatermarkOptions.svelte';
  import ImmichOptions from './export/ImmichOptions.svelte';
  import { exportSettings } from './export/exportSettings.svelte';
  import {
    baseOptions,
    COLOR_SPACES,
    ensureLibraryLoaded,
    formatLabel,
    formInvalid,
    immichOptions
  } from './export/settings';

  const form = $derived(exportSettings.form);
  const destination = $derived(exportSettings.destination);

  $effect(() => {
    if (destination === 'immich') ensureLibraryLoaded(exportSettings.form);
  });

  let isLoading = $derived(
    destination === 'download' ? editor.exporting : editor.exportingToImmich
  );
  let label = $derived(formatLabel(form.format));
  let buttonLabel = $derived(
    destination === 'download' ? `Export ${label}` : `Upload ${label} to Immich`
  );
  let busyLabel = $derived(destination === 'download' ? 'Exporting…' : 'Uploading…');
  let result = $derived(destination === 'download' ? editor.lastDownload : editor.lastUpload);
  let proofMismatch = $derived(editor.isProofing && editor.proofSpace !== form.colorSpace);
  function spaceLabel(space: string): string {
    return COLOR_SPACES.find((c) => c.value === space)?.label ?? space;
  }
  let crop = $derived(
    editor.meta
      ? croppedOutputSize(editor.edits.geometry, editor.meta.source_w, editor.meta.source_h)
      : null
  );
  let nameExample = $derived(
    editor.asset
      ? {
          original: editor.asset.originalFileName,
          date: captureDate(editor.asset),
          position: 1,
          total: 1
        }
      : null
  );
  let invalid = $derived(formInvalid(form));
</script>

<div class="flex flex-col gap-1">
  <DestinationToggle bind:value={exportSettings.destination} />

  <ExportSection title="File">
    <FormatOptions bind:form={exportSettings.form} />
    {#if proofMismatch}
      <Notice
        color="warning"
        message={`Soft proofing ${spaceLabel(editor.proofSpace)}, exporting ${spaceLabel(form.colorSpace)}`}
      >
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-6 panel-action"
          onclick={() => (exportSettings.form.colorSpace = editor.proofSpace)}
        >
          Export {spaceLabel(editor.proofSpace)}
        </Button>
      </Notice>
    {/if}
  </ExportSection>

  <ExportSection title="Size">
    <ResizeOptions bind:form={exportSettings.form} {crop} />
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

  {#if result}
    <Notice
      color={result.kind === 'success'
        ? 'success'
        : result.kind === 'duplicate'
          ? 'warning'
          : 'danger'}
      message={result.message}
    >
      {#if result.kind === 'error'}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="h-6 panel-action"
          leadingIcon={mdiRefresh}
          onclick={() =>
            void (destination === 'download' ? editor.retryExport() : editor.retryUpload())}
          disabled={isLoading}
        >
          Retry
        </Button>
      {/if}
    </Notice>
  {/if}

  {#if destination === 'immich' && editor.lastWarnings.length > 0}
    <ul
      class="space-y-0.5 rounded-sm border border-warning-500/40 bg-warning-950/40 px-2 py-1.5 text-[10px] leading-snug text-warning-100"
    >
      {#each editor.lastWarnings as w (w)}
        <li class="flex items-start gap-1.5">
          <Icon icon={mdiAlertOutline} size="12px" class="mt-0.5 shrink-0" aria-hidden="true" />
          <span>{w}</span>
        </li>
      {/each}
    </ul>
  {/if}

  <div class="sticky bottom-0 -mx-3 border-t border-hairline bg-light px-3 py-1.5">
    <Button
      size="small"
      color="primary"
      fullWidth
      loading={isLoading}
      leadingIcon={destination === 'download' ? mdiExport : mdiCloudUpload}
      disabled={isLoading || !editor.assetId || invalid}
      onclick={() => {
        if (destination === 'download') void editor.onExport(baseOptions(form));
        else void editor.onUploadToImmich(immichOptions(form));
      }}
    >
      {isLoading ? busyLabel : buttonLabel}
    </Button>
  </div>
</div>
