<script lang="ts">
  import { Button } from '@immich/ui';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';

  const exporter = new BulkExporter();
</script>

<div class="flex flex-col gap-1 px-3 py-1.5">
  <div class="text-[11px] text-dark/65 select-none">
    {exporter.summary}
  </div>

  <ExportForm
    scope={exporter.scope}
    bind:replaceOriginal={exporter.replaceOriginal}
    downloadLabel="Download ZIP"
  />

  <Button
    size="small"
    color="primary"
    fullWidth
    loading={exporter.busy}
    leadingIcon={exporter.icon}
    disabled={exporter.disabled}
    onclick={() => void exporter.submit()}
  >
    {exporter.label}
  </Button>

  <p class="text-[10px] leading-snug text-dark/65">
    Runs as a background job. Track progress in the Jobs panel.
  </p>
</div>
