<script lang="ts">
  import BulkExport from '#lib/components/browse/bulk/BulkExport.svelte';
  import BulkExportFooter from '#lib/components/browse/bulk/BulkExportFooter.svelte';
  import ExportSheet from '#lib/panels/export/ExportSheet.svelte';
  import { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';

  let {
    open,
    title,
    onOpenChange,
    onExited
  }: {
    open: boolean;
    title: string;
    onOpenChange: (open: boolean) => void;
    onExited?: () => void;
  } = $props();

  const exporter = new BulkExporter(() => onOpenChange(false));
</script>

{#snippet footer()}
  <BulkExportFooter {exporter} />
{/snippet}

<ExportSheet {open} {onOpenChange} {onExited} {title} {exporter} shareHints={false} {footer}>
  <BulkExport {exporter} />
</ExportSheet>
