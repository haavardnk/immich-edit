<script lang="ts">
  import { Button } from '@immich/ui';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import ExportResult from '#lib/panels/export/ExportResult.svelte';
  import ProofMismatchNotice from '#lib/panels/export/ProofMismatchNotice.svelte';
  import { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';

  const exporter = new EditorExporter();
</script>

{#snippet proofNotice()}
  <ProofMismatchNotice />
{/snippet}

<div class="flex flex-col gap-1">
  <ExportForm
    scope={exporter.scope}
    crop={exporter.crop}
    bind:replaceOriginal={exporter.replaceOriginal}
    fileNotice={proofNotice}
  />

  <ExportResult result={exporter.result} busy={exporter.busy} onRetry={exporter.retry} />

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
