<script lang="ts">
  import ExportActionBar from '#lib/panels/export/ExportActionBar.svelte';
  import type { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';

  let { exporter, stacked = false }: { exporter: BulkExporter; stacked?: boolean } = $props();
</script>

{#snippet summary()}
  <span class="truncate text-xs text-dark touch:text-sm">{exporter.summary}</span>
  {#if exporter.note}
    <span class="text-[10px] text-dark/65 touch:text-xs">{exporter.note}</span>
  {/if}
{/snippet}

<ExportActionBar
  label={exporter.label}
  icon={exporter.icon}
  busy={exporter.busy}
  disabled={exporter.disabled}
  onStart={() => void exporter.submit()}
  {summary}
  {stacked}
/>
