<script lang="ts">
  import ExportFormStack from '#lib/panels/export/ExportFormStack.svelte';
  import ExportedFileActions from '#lib/panels/export/ExportedFileActions.svelte';
  import ExportedFileCard from '#lib/panels/export/ExportedFileCard.svelte';
  import { EditorExporter } from '#lib/panels/export/editorExport.svelte.ts';
  import { Button } from '@immich/ui';

  const exporter = new EditorExporter();
</script>

<div class="flex flex-col gap-1">
  <ExportFormStack {exporter} folded={exporter.folded}>
    {#if exporter.file}
      {@const file = exporter.file}
      <div class="mt-1.5 flex flex-col gap-1">
        <ExportedFileCard {file}>
          {#snippet actions()}
            <ExportedFileActions
              {file}
              uploading={exporter.uploading}
              onSave={exporter.save}
              onUpload={exporter.uploadFile}
            />
          {/snippet}
        </ExportedFileCard>
      </div>
    {/if}
  </ExportFormStack>

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
