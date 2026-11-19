<script lang="ts">
  import type { ExportPreset } from '#lib/api/exportPresets.ts';
  import Notice from '#lib/components/Notice.svelte';
  import TruncationTooltip from '#lib/components/TruncationTooltip.svelte';
  import { Badge, Button, Checkbox, IconButton } from '@immich/ui';
  import { mdiContentSaveOutline, mdiDeleteOutline } from '@mdi/js';
  import { mergeProps } from 'bits-ui';
  import { exportPresets } from './exportPresets.svelte';
  import type { PresetSelection } from './presetSelection.svelte';
  import { exportSettings } from './exportSettings.svelte';
  import { albumNames, ensureLibraryLoaded, restoreExportForm } from './settings';
  import { SAVE_ONLY_REASON, saveOnly } from './shareFile';
  import { presetSummary } from './summary';

  let {
    selection,
    onApply,
    shareHints = false
  }: {
    selection: PresetSelection;
    onApply: (preset: ExportPreset) => void;
    shareHints?: boolean;
  } = $props();

  const uid = $props.id();
  let name = $state('');

  const rows = $derived(
    exportPresets.presets.map((preset) => {
      const form = restoreExportForm(preset.form);
      const immich = preset.destination === 'immich';
      return {
        preset,
        summary: presetSummary(form, immich ? { albums: albumNames(form.albumIds) } : null),
        saveOnly: shareHints && !immich && saveOnly(form.format)
      };
    })
  );

  const anySaveOnly = $derived(rows.some((row) => row.saveOnly));

  const anyImmich = $derived(
    exportPresets.presets.some((preset) => preset.destination === 'immich')
  );

  $effect(() => {
    if (anyImmich) ensureLibraryLoaded(exportSettings.form);
  });

  async function save(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    const trimmed = name.trim();
    if (!trimmed) return;
    await exportPresets.save(trimmed);
    name = '';
  }
</script>

<ul class="py-1" aria-label="Export presets">
  {#each rows as { preset, summary, saveOnly: onlySave } (preset.id)}
    {@const checkId = `${uid}-${preset.id}`}
    {@const summaryId = `${checkId}-summary`}
    <li
      class="flex items-center gap-2.5 ps-3 pe-1 touch:min-h-14 touch:gap-3 touch:ps-4 {selection.has(
        preset.id
      )
        ? 'bg-primary/10'
        : 'hover:bg-light-950/5'}"
    >
      <Checkbox
        id={checkId}
        size="tiny"
        aria-label="Export with {preset.name}"
        aria-describedby={summaryId}
        bind:checked={() => selection.has(preset.id), (on) => selection.toggle(preset.id, on)}
      />
      <TruncationTooltip text="{preset.name}: {summary}">
        {#snippet child({ props })}
          <label
            {...mergeProps(props, {
              for: checkId,
              tabindex: -1,
              class: 'min-w-0 flex-1 cursor-pointer py-1.5 select-none'
            })}
          >
            <span data-truncate class="block truncate text-xs text-dark touch:text-[15px]">
              {preset.name}
            </span>
            <span
              id={summaryId}
              data-truncate
              class="block truncate text-[10px] text-dark/60 touch:text-xs"
            >
              {summary}
            </span>
          </label>
        {/snippet}
      </TruncationTooltip>
      {#if onlySave}
        <Badge size="tiny" color="warning" class="shrink-0">Save only</Badge>
      {/if}
      <Button
        size="tiny"
        variant="ghost"
        color="secondary"
        class="shrink-0 touch:min-h-11 touch:text-sm"
        aria-label="Apply {preset.name}"
        onclick={() => onApply(preset)}
      >
        Apply
      </Button>
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        shape="round"
        class="shrink-0 text-dark/50 hover:text-red-400 touch:size-11"
        icon={mdiDeleteOutline}
        title="Delete"
        aria-label="Delete export preset {preset.name}"
        onclick={() => void exportPresets.remove(preset)}
      />
    </li>
  {:else}
    <li class="px-3 py-1.5 text-xs text-dark/50 touch:px-4 touch:py-3 touch:text-sm">
      {exportPresets.loaded ? 'No saved presets' : 'Loading…'}
    </li>
  {/each}
</ul>
{#if anySaveOnly}
  <Notice color="info" message={SAVE_ONLY_REASON} class="mx-3 mb-2" />
{/if}
<form class="flex items-center gap-1 border-t border-dark/10 p-2" onsubmit={save}>
  <input
    bind:value={name}
    maxlength={80}
    placeholder="Save current as…"
    aria-label="Export preset name"
    class="min-w-0 flex-1 rounded border border-dark/15 bg-transparent px-2 py-1 text-xs touch:h-11 touch:text-[15px]"
  />
  <IconButton
    type="submit"
    size="small"
    variant="ghost"
    color="secondary"
    shape="round"
    class="touch:size-11"
    icon={mdiContentSaveOutline}
    title="Save (overwrites a preset with the same name)"
    aria-label="Save export preset"
    disabled={!name.trim()}
  />
</form>
