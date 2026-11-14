<script lang="ts">
  import { Button } from '@immich/ui';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';

  let { onStarted }: { onStarted: () => void } = $props();

  const exporter = new BulkExporter(() => onStarted());
</script>

<div class="flex flex-col gap-1 px-3 py-1.5">
  <ExportForm
    scope={exporter.scope}
    bind:replaceOriginal={exporter.replaceOriginal}
    downloadLabel={exporter.downloadLabel}
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
</div>
