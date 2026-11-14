<script lang="ts">
  import { Button } from '@immich/ui';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import ExportResult from '#lib/panels/export/ExportResult.svelte';
  import ProofMismatchNotice from '#lib/panels/export/ProofMismatchNotice.svelte';
  import { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import ExportedFileCard from '#lib/panels/export/ExportedFileCard.svelte';
  import { exportSettings } from '#lib/panels/export/exportSettings.svelte.ts';

  const exporter = new EditorExporter();
</script>

{#snippet proofNotice()}
  <ProofMismatchNotice />
{/snippet}

<div class="flex flex-col gap-1">
  <ExportForm
    scope={exporter.scope}
    crop={exporter.crop}
    folded={exporter.folded}
    bind:replaceOriginal={exporter.replaceOriginal}
    fileNotice={proofNotice}
  />

  <ExportResult result={exporter.result} busy={exporter.busy} onRetry={exporter.retry} />

  {#if exporter.file}
    <div class="mt-1.5 flex flex-col gap-1">
      <ExportedFileCard
        file={exporter.file}
        uploading={exporter.uploading}
        onSave={exporter.save}
        onUpload={exporter.uploadFile}
      />
    </div>
  {/if}

  {#if exportSettings.destination === 'download'}
    <CheckboxRow
      label="Save automatically"
      class="mt-1 touch:h-11 touch:text-sm"
      checked={exportSettings.saveAutomatically}
      onChange={(checked) => (exportSettings.saveAutomatically = checked)}
    >
      <span class="ms-1 text-dark/50">on this device</span>
    </CheckboxRow>
  {/if}

  <div class="sticky bottom-0 -mx-3 border-t border-hairline bg-editor-panel px-3 py-1.5">
    <Button
      size="small"
      color="primary"
      fullWidth
      loading={exporter.busy}
      leadingIcon={exporter.icon}
      disabled={exporter.disabled}
      onclick={exporter.start}
    >
      {exporter.label}
    </Button>
  </div>
</div>
