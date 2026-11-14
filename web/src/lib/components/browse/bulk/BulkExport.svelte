<script lang="ts">
  import Notice from '#lib/components/Notice.svelte';
  import ExportForm from '#lib/panels/export/ExportForm.svelte';
  import ExportPresetList from '#lib/panels/export/ExportPresetList.svelte';
  import ExportSection from '#lib/panels/export/ExportSection.svelte';
  import { BulkExporter } from '#lib/panels/export/bulkExport.svelte.ts';
  import { exportPresets } from '#lib/panels/export/exportPresets.svelte.ts';
  import { Button } from '@immich/ui';
  import { onMount } from 'svelte';

  let { onStarted, folded = false }: { onStarted: () => void; folded?: boolean } = $props();

  const exporter = new BulkExporter(() => onStarted());

  onMount(() => {
    if (!exportPresets.loaded) void exportPresets.load();
  });
</script>

<div class="flex flex-col gap-1 px-3 pt-1.5">
  <ExportSection title="Presets">
    <div class="-mx-3">
      <ExportPresetList selection={exporter.presets} onApply={exportPresets.apply} />
    </div>
  </ExportSection>

  {#if exportPresets.presets.length > 0}
    <div
      class="flex min-h-7 items-center gap-2 border-t border-hairline pt-1.5 touch:min-h-11 touch:pt-3"
    >
      <h3
        class="text-[10px] font-medium tracking-wider text-dark/65 uppercase select-none touch:text-[11px]"
      >
        Settings
      </h3>
      <p class="min-w-0 flex-1 text-[10px] text-dark/60 touch:text-xs">
        {exporter.usingPresets ? 'Off while presets are ticked' : 'Used when no preset is ticked'}
      </p>
      {#if exporter.usingPresets}
        <Button
          size="tiny"
          variant="ghost"
          color="secondary"
          class="shrink-0 touch:min-h-11 touch:text-sm"
          onclick={exporter.presets.clear}
        >
          Use settings
        </Button>
      {/if}
    </div>
  {/if}

  {#if !exporter.usingPresets}
    <ExportForm
      scope={exporter.scope}
      bind:replaceOriginal={exporter.replaceOriginal}
      downloadLabel={exporter.downloadLabel}
      {folded}
    />
  {/if}

  {#if exporter.zipNotice}
    <Notice
      color="info"
      message="Saves one ZIP instead of sharing. Share from the editor works one photo at a time."
      class="mt-1"
    />
  {/if}

  <div
    class="sticky bottom-0 -mx-3 mt-1 flex items-center gap-3 border-t border-hairline bg-light-100 px-3 py-2 touch:py-3"
  >
    <div class="min-w-0 flex-1 leading-tight">
      <p class="truncate text-xs text-dark touch:text-sm">{exporter.summary}</p>
      {#if exporter.note}
        <p class="text-[10px] text-dark/60 touch:text-xs">{exporter.note}</p>
      {/if}
    </div>
    <Button
      size="small"
      color="primary"
      class="shrink-0 touch:min-h-11"
      loading={exporter.busy}
      leadingIcon={exporter.icon}
      disabled={exporter.disabled}
      onclick={() => void exporter.submit()}
    >
      {exporter.label}
    </Button>
  </div>
</div>
