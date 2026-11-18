<script lang="ts">
  import { presetThumbUrl, type Preset } from '#lib/api/presets.ts';
  import BottomPanelResizeHandle from '#lib/components/shell/BottomPanelResizeHandle.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { presetStrip } from '#lib/stores/presetStrip.svelte.ts';
  import { presets } from '#lib/stores/presets.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { onDestroy } from 'svelte';
  import PresetApplyOptions from './ApplyOptions.svelte';
  import PresetActions from './PresetActions.svelte';
  import PresetGroups from './PresetGroups.svelte';
  import PresetTile from './PresetTile.svelte';
  import SavePresetDialog from './SavePresetDialog.svelte';
  import { scrollSideways } from '#lib/input/sidewaysWheel.ts';

  let dialog = $state<{ preset?: Preset } | null>(null);
  const tileWidth = $derived(Math.round(ui.filmstripHeight * 1.5));

  $effect(presetStrip.ensureLoaded);

  onDestroy(presetStrip.endPreview);
</script>

{#snippet actions()}
  <PresetActions oncreate={() => (dialog = {})} />
{/snippet}

<section
  aria-label="Presets"
  class="relative flex flex-none gap-3 border-t border-light-950/12 bg-editor-chrome px-2 py-1.5 dark:shadow-filmstrip"
>
  <BottomPanelResizeHandle label="Resize presets" />
  <PresetGroups {actions} />
  <ul
    class="flex min-w-0 flex-1 gap-1.5 overflow-x-auto scrollbar-hidden"
    style:height="{ui.filmstripHeight}px"
    onwheel={scrollSideways}
  >
    {#if presets.presets.length === 0}
      <li class="flex items-center ps-2 text-xs text-dark/65">No presets yet</li>
    {:else if presetStrip.shown.length === 0}
      <li class="flex items-center ps-2 text-xs text-dark/65">No presets match</li>
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
  </ul>
  <div
    class="flex w-64 flex-none flex-col justify-center border-l border-hairline pl-3"
    style:height="{ui.filmstripHeight}px"
  >
    <PresetApplyOptions
      bind:options={presetStrip.applyOptions}
      inline
      amount={presetStrip.amount}
      note={presetStrip.live ? `Blending ${presetStrip.live.name}` : null}
      onamount={presetStrip.setAmount}
      onamountcommit={presetStrip.commitAmount}
    />
  </div>
</section>

{#if dialog}
  <SavePresetDialog preset={dialog.preset} onClose={() => (dialog = null)} />
{/if}
