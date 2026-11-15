<script lang="ts">
  import Popover from '#lib/components/Popover.svelte';
  import { reveal } from '#lib/motion/reveal.ts';
  import { countOf } from '#lib/utils/plural.ts';
  import { Button, Icon } from '@immich/ui';
  import { mdiLayersTripleOutline, mdiMenuDown } from '@mdi/js';
  import EarlierExport from './EarlierExport.svelte';
  import type { EarlierExports, EarlierQuery } from './earlierExports.svelte';
  import ExportPresetList from './ExportPresetList.svelte';
  import { exportPresets } from './exportPresets.svelte';
  import type { PresetSelection } from './presetSelection.svelte';

  let {
    selection,
    busy,
    onExport,
    earlier,
    earlierQuery = null,
    replaceReason = null
  }: {
    selection: PresetSelection;
    busy: boolean;
    onExport: () => Promise<boolean>;
    earlier?: EarlierExports;
    earlierQuery?: EarlierQuery | null;
    replaceReason?: string | null;
  } = $props();

  let open = $state(false);

  function onOpenChange(next: boolean): void {
    open = next;
    if (next && !exportPresets.loaded) void exportPresets.load();
  }

  async function run(): Promise<void> {
    if (await onExport()) open = false;
  }
</script>

<Popover
  {open}
  {onOpenChange}
  align="start"
  appearance="panel"
  contentClass="w-md"
  sheetClass="p-0"
  title="Export presets"
>
  {#snippet trigger(props)}
    <Button
      size="tiny"
      variant="ghost"
      color="secondary"
      class="gap-1 self-start font-normal text-dark/65 touch:min-h-11 touch:text-sm"
      title=""
      {...props}
    >
      Export presets
      <Icon icon={mdiMenuDown} size="16px" class="shrink-0" aria-hidden="true" />
    </Button>
  {/snippet}
  <div class="px-3 pt-2 text-xs font-medium text-dark touch:hidden">Export presets</div>
  <ExportPresetList
    {selection}
    shareHints
    onApply={(preset) => {
      exportPresets.apply(preset);
      open = false;
    }}
  />
  {#if earlier && earlierQuery}
    <EarlierExport {earlier} query={earlierQuery} {replaceReason} class="mx-3 mb-2 touch:mx-4" />
  {/if}
  {#if selection.picked.length > 0}
    <div
      transition:reveal
      class="sticky bottom-0 flex items-center gap-2 border-t border-dark/10 bg-light-100 px-3 py-2 touch:px-4 touch:py-3"
    >
      <span class="min-w-0 flex-1 text-[11px] leading-tight text-dark/65 touch:text-xs">
        {selection.note}
      </span>
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="touch:min-h-11 touch:text-sm"
        onclick={selection.clear}
      >
        Clear
      </Button>
      <Button
        size="tiny"
        color="primary"
        class="touch:min-h-11 touch:text-sm"
        leadingIcon={mdiLayersTripleOutline}
        loading={busy}
        disabled={busy || selection.tooMany}
        onclick={() => void run()}
      >
        Export with {countOf(selection.picked.length, 'preset')}
      </Button>
    </div>
  {/if}
</Popover>
