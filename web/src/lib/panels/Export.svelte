<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import Notice from '$lib/components/Notice.svelte';
  import { Button, Icon } from '@immich/ui';
  import { mdiExport, mdiCloudUpload, mdiRefresh, mdiAlertOutline } from '@mdi/js';
  import { EXTENSION_BY_FORMAT } from '$lib/api/export';
  import { captureDate, templateError } from '$lib/filenameTemplate';
  import { croppedOutputSize } from '$lib/utils/geom';
  import { fmtDim } from '$lib/utils/exif';
  import DestinationToggle from './export/DestinationToggle.svelte';
  import FilenameTemplateField from './export/FilenameTemplateField.svelte';
  import FormatOptions from './export/FormatOptions.svelte';
  import ImmichOptions from './export/ImmichOptions.svelte';
  import { exportSettings } from './export/exportSettings.svelte';
  import {
    baseOptions,
    COLOR_SPACES,
    ensureLibraryLoaded,
    formatLabel,
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
  let outputSize = $derived.by(() => {
    const meta = editor.meta;
    if (!meta) return null;
    const size = croppedOutputSize(editor.edits.geometry, meta.source_w, meta.source_h);
    return `${fmtDim(size.w, size.h)} px`;
  });
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
  let nameInvalid = $derived(templateError(form.filenameTemplate) !== null);
</script>

<div class="flex flex-col gap-1">
  <DestinationToggle bind:value={exportSettings.destination} />

  <FormatOptions bind:form={exportSettings.form} {outputSize} />

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

  <div class="flex flex-col gap-1 border-t border-hairline pt-1.5">
    <FilenameTemplateField
      bind:value={exportSettings.form.filenameTemplate}
      example={nameExample}
      extension={EXTENSION_BY_FORMAT[form.format]}
    />
    {#if destination === 'immich'}
      <ImmichOptions bind:form={exportSettings.form} />
    {/if}
  </div>

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
      disabled={isLoading || !editor.assetId || nameInvalid}
      onclick={() => {
        if (destination === 'download') void editor.onExport(baseOptions(form));
        else void editor.onUploadToImmich(immichOptions(form));
      }}
    >
      {isLoading ? busyLabel : buttonLabel}
    </Button>
  </div>
</div>
