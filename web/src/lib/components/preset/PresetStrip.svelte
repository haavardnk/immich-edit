<script lang="ts">
  import { presetThumbUrl, type Preset } from '#lib/api/presets.ts';
  import BottomPanelResizeHandle from '#lib/components/shell/BottomPanelResizeHandle.svelte';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { onDestroy } from 'svelte';
  import PresetApplyOptions from './ApplyOptions.svelte';
  import PresetCreateColumn from './PresetCreateColumn.svelte';
  import PresetGroups from './PresetGroups.svelte';
  import PresetTile from './PresetTile.svelte';
  import SavePresetDialog from './SavePresetDialog.svelte';
  import { scrollSideways } from '#lib/input/sidewaysWheel.ts';

  let dialog = $state<{ preset?: Preset } | null>(null);
  const tileWidth = $derived(Math.round(ui.filmstripHeight * 1.5));

  $effect(presetStrip.ensureLoaded);

  onDestroy(presetStrip.endPreview);
</script>

{#snippet items()}
  <PresetCreateColumn oncreate={() => (dialog = {})} />
  {#if presets.presets.length === 0}
    <li class="flex items-center ps-2 text-xs text-dark/65">
      No presets yet. Save the current look or import a preset file.
    </li>
  {:else if presetStrip.shown.length === 0}
    <li class="flex items-center ps-2 text-xs text-dark/65">No presets match.</li>
  {/if}
  {#each presetStrip.shown as preset (preset.id)}
    <PresetTile
      {preset}
      src={editor.assetId && editor.initialised
        ? presetThumbUrl(editor.assetId, preset, editor.savedHash)
        : null}
      width={tileWidth}
      disabled={!editor.initialised}
      onedit={() => (dialog = { preset })}
    />
  {/each}
{/snippet}

{#snippet options()}
  <PresetApplyOptions
    bind:options={presetStrip.applyOptions}
    inline
    amount={presetStrip.amount}
    note={presetStrip.live ? `Blending ${presetStrip.live.name}` : null}
    onamount={presetStrip.setAmount}
    onamountcommit={presetStrip.commitAmount}
  />
{/snippet}

{#if layout.mode === 'touch'}
  <section aria-label="Presets" class="flex flex-col gap-2 pt-1 pb-3">
    {#if presetStrip.filterable}
      <PresetGroups chips />
    {/if}
    <ul
      class="flex flex-wrap gap-2 px-3 scrollbar-hidden [[data-arrangement=stack]_&]:flex-nowrap [[data-arrangement=stack]_&]:overflow-x-auto"
    >
      {@render items()}
    </ul>
    <div class="px-4">
      {@render options()}
    </div>
  </section>
{:else}
  <section
    aria-label="Presets"
    class="relative flex flex-none gap-3 border-t border-light-950/12 bg-editor-chrome px-2 py-1.5 dark:shadow-filmstrip"
  >
    <BottomPanelResizeHandle label="Resize presets" />
    {#if presets.presets.length > 0}
      <PresetGroups />
    {/if}
    <ul
      class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto scrollbar-hidden"
      style:height="{ui.filmstripHeight}px"
      onwheel={scrollSideways}
    >
      {@render items()}
    </ul>
    <div
      class="flex w-64 flex-none flex-col justify-center border-l border-hairline pl-3"
      style:height="{ui.filmstripHeight}px"
    >
      {@render options()}
    </div>
  </section>
{/if}

{#if dialog}
  <SavePresetDialog preset={dialog.preset} onClose={() => (dialog = null)} />
{/if}
