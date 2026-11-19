<script lang="ts">
  import BulkExportPanel from '#lib/components/browse/bulk/BulkExportPanel.svelte';
  import { bulkExportDock } from '#lib/browse/bulk/bulkExportDock.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { reveal } from '#lib/motion/reveal.ts';
</script>

{#if bulkExportDock.open && layout.mode === 'desktop'}
  <div
    class="flex h-full flex-none max-md:fixed max-md:inset-y-0 max-md:right-0 max-md:z-40 max-md:shadow-popover"
    transition:reveal={{ axis: 'x' }}
  >
    <BulkExportPanel />
  </div>
{/if}
