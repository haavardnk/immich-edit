<script lang="ts">
  import { onDestroy } from 'svelte';
  import FileImportButton from '$lib/components/FileImportButton.svelte';
  import BottomPanelResizeHandle from '$lib/components/shell/BottomPanelResizeHandle.svelte';
  import PresetApplyOptions from './ApplyOptions.svelte';
  import PresetTile from './PresetTile.svelte';
  import SavePresetDialog from './SavePresetDialog.svelte';
  import { presetThumbUrl, type Preset } from '$lib/api/presets';
  import type { ApplyPresetOptions } from '$lib/api/jobs';
  import { LOOK_AMOUNT_FULL } from '$lib/edits/lookAmount';
  import { editor } from '$lib/stores/editor.svelte';
  import { presets } from '$lib/stores/presets.svelte';
  import { ui } from '$lib/stores/ui.svelte';
  import { Icon } from '@immich/ui';
  import { mdiPlus, mdiUpload } from '@mdi/js';

  let applyOptions = $state<ApplyPresetOptions>({
    includeGeometry: false,
    includeMasks: false,
    amount: LOOK_AMOUNT_FULL
  });
  let pickedGroup = $state<string | null>(null);
  let dialog = $state<{ preset?: Preset } | null>(null);
  const tileWidth = $derived(Math.round(ui.filmstripHeight * 1.5));
  const createButtonClass =
    'flex w-full items-center justify-center gap-1 rounded-sm border border-dashed border-white/20 text-[11px] text-dark/65 outline-none focus-visible:ring-2 focus-visible:ring-primary enabled:hover:border-white/40 enabled:hover:text-dark disabled:cursor-not-allowed disabled:opacity-50';

  const groups = $derived(
    presets.grouped
      .map((g) => g.group)
      .sort((a, b) => Number(a === '') - Number(b === '') || a.localeCompare(b))
  );
  const groupOptions = $derived([
    { value: null, label: 'All' },
    ...groups.map((g) => ({ value: g, label: g || 'Ungrouped' }))
  ]);
  const group = $derived(pickedGroup !== null && groups.includes(pickedGroup) ? pickedGroup : null);
  const shown = $derived(
    group === null ? presets.presets : presets.presets.filter((p) => (p.group_name ?? '') === group)
  );

  $effect(() => {
    if (!presets.loaded && !presets.loading) void presets.load();
  });

  onDestroy(editor.endPresetPreview);

  function preview(preset: Preset): void {
    editor.previewPreset({ manifest: preset.manifest, opts: $state.snapshot(applyOptions) });
  }

  function apply(preset: Preset): void {
    void editor.applyPreset(preset.manifest, $state.snapshot(applyOptions), preset.name);
  }

  function onWheel(e: WheelEvent): void {
    const row = e.currentTarget as HTMLElement;
    if (e.deltaX !== 0 || e.deltaY === 0 || e.ctrlKey || e.metaKey) return;
    e.preventDefault();
    row.scrollLeft += e.deltaY;
  }
</script>

<section
  aria-label="Presets"
  class="relative flex flex-none gap-3 border-t border-white/12 bg-editor-chrome px-2 py-1.5 shadow-filmstrip"
>
  <BottomPanelResizeHandle label="Resize presets" />
  {#if groups.length > 1}
    <nav
      aria-label="Preset groups"
      class="flex w-20 flex-none flex-col gap-px overflow-y-auto scrollbar-hidden"
      style:height="{ui.filmstripHeight}px"
    >
      {#each groupOptions as option (option.value)}
        <button
          type="button"
          class="flex-none truncate rounded-sm px-1.5 py-0.5 text-left text-[11px] outline-none focus-visible:ring-2 focus-visible:ring-primary {group ===
          option.value
            ? 'bg-white/10 text-dark'
            : 'text-dark/60 hover:text-dark'}"
          title={option.label}
          aria-pressed={group === option.value}
          onclick={() => (pickedGroup = option.value)}>{option.label}</button
        >
      {/each}
    </nav>
  {/if}
  <ul
    class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto scrollbar-hidden"
    style:height="{ui.filmstripHeight}px"
    onwheel={onWheel}
  >
    <li class="flex w-24 flex-none flex-col gap-0.5">
      <button
        type="button"
        class="{createButtonClass} min-h-0 flex-1"
        title={editor.hasEdits ? 'Save current as preset' : 'Edit the photo to save a preset'}
        aria-label="Save current as preset"
        disabled={!editor.hasEdits}
        onclick={() => (dialog = {})}
      >
        <Icon icon={mdiPlus} size="14px" aria-hidden="true" />
        <span class="truncate">{editor.hasEdits ? 'New preset' : 'No edits yet'}</span>
      </button>
      <FileImportButton
        accept=".json,application/json"
        label="Import presets"
        onfile={presets.importFile}
      >
        {#snippet button({ busy, open })}
          <button
            type="button"
            class="{createButtonClass} h-6 flex-none"
            title="Import presets from a file"
            aria-label="Import presets"
            disabled={busy}
            onclick={open}
          >
            <Icon icon={mdiUpload} size="14px" aria-hidden="true" />
            <span class="truncate">{busy ? 'Importing…' : 'Import'}</span>
          </button>
        {/snippet}
      </FileImportButton>
    </li>
    {#if presets.presets.length === 0}
      <li class="flex items-center ps-2 text-xs text-dark/65">
        No presets yet. Save the current look or import a preset file.
      </li>
    {/if}
    {#each shown as preset (preset.id)}
      <PresetTile
        {preset}
        src={editor.assetId && editor.initialised
          ? presetThumbUrl(editor.assetId, preset, editor.savedHash)
          : null}
        width={tileWidth}
        disabled={!editor.initialised}
        onpreview={() => preview(preset)}
        onendpreview={editor.endPresetPreview}
        onapply={() => apply(preset)}
        onedit={() => (dialog = { preset })}
      />
    {/each}
  </ul>
  <div
    class="flex w-64 flex-none flex-col justify-center border-l border-hairline pl-3"
    style:height="{ui.filmstripHeight}px"
  >
    <PresetApplyOptions bind:options={applyOptions} inline />
  </div>
</section>

{#if dialog}
  <SavePresetDialog preset={dialog.preset} onClose={() => (dialog = null)} />
{/if}
