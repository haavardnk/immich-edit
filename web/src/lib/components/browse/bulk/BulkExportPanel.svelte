<script lang="ts">
  import BulkExport from '#lib/components/browse/bulk/BulkExport.svelte';
  import BulkExportFooter from '#lib/components/browse/bulk/BulkExportFooter.svelte';
  import ExportPresetMenu from '#lib/panels/export/ExportPresetMenu.svelte';
  import { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';
  import {
    BULK_EXPORT_BUTTON,
    BULK_EXPORT_PANEL,
    bulkExportDock
  } from '#lib/browse/bulk/bulkExportDock.svelte.ts';
  import { bulkScope } from '#lib/browse/bulk/bulkScope.svelte.ts';
  import { browseView } from '#lib/stores/browseView.svelte.ts';
  import { jobs } from '#lib/stores/jobs.svelte.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { IconButton } from '@immich/ui';
  import { mdiClose } from '@mdi/js';
  import { onDestroy, onMount, tick } from 'svelte';
  import { on } from 'svelte/events';

  let panel = $state<HTMLElement | null>(null);

  const exporter = new BulkExporter(bulkExportDock.close);

  onMount(() => panel?.focus());

  onDestroy(bulkExportDock.close);

  $effect(() => {
    if (jobs.open || browseView.filtersOpen || !bulkScope.selecting || bulkScope.count === 0) {
      bulkExportDock.close();
    }
  });

  async function close(): Promise<void> {
    bulkExportDock.close();
    await tick();
    document.getElementById(BULK_EXPORT_BUTTON)?.focus();
  }

  function closeOnEscape(node: HTMLElement): () => void {
    return on(node, 'keydown', (event) => {
      if (event.key !== 'Escape' || event.defaultPrevented) return;
      event.preventDefault();
      event.stopPropagation();
      void close();
    });
  }
</script>

<aside
  bind:this={panel}
  id={BULK_EXPORT_PANEL}
  aria-labelledby="bulk-export-title"
  tabindex="-1"
  class="flex h-full w-96 max-w-[92vw] flex-col border-s border-hairline bg-light-50 outline-none"
  {@attach closeOnEscape}
>
  <header class="flex h-14 flex-none items-center gap-2 border-b border-hairline ps-4 pe-2">
    <h2 id="bulk-export-title" class="text-[15px] font-semibold text-light-950">Export</h2>
    <span class="text-xs whitespace-nowrap text-dark/65">{countOf(bulkScope.count, 'photo')}</span>
    <div class="ms-auto flex items-center gap-1">
      <div class="flex">
        <ExportPresetMenu {exporter} shareHints={false} align="end" />
      </div>
      <IconButton
        size="small"
        variant="ghost"
        color="secondary"
        icon={mdiClose}
        title="Close"
        aria-label="Close export"
        onclick={() => void close()}
      />
    </div>
  </header>
  <div class="min-h-0 flex-1 overflow-y-auto px-3 py-2 scrollbar-hidden">
    <BulkExport {exporter} />
  </div>
  <footer class="flex-none border-t border-hairline p-3">
    <BulkExportFooter {exporter} stacked />
  </footer>
</aside>
