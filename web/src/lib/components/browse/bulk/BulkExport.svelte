<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import EarlierExport from '#lib/panels/export/EarlierExport.svelte';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import type { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';

  let { exporter }: { exporter: BulkExporter } = $props();
</script>

<div class="flex flex-col gap-1">
  <ExportForm scope={exporter.scope} bind:replaceOriginal={exporter.replaceOriginal} />

  <EarlierExport
    earlier={exporter.earlier}
    query={exporter.earlierQuery}
    replaceReason={exporter.scope.conflictReason}
  />

  {#if exporter.zipNotice}
    <Notice
      color="info"
      message="Saves one ZIP instead of sharing. Share from the editor works one photo at a time."
      class="mt-1"
    />
  {/if}
</div>
