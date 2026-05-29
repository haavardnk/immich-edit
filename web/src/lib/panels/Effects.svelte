<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { NEUTRAL_EFFECTS } from '$lib/types/edits';

  const vignetteInactive = $derived(editor.edits.effects.vignette_amount === 0);
  const grainInactive = $derived(editor.edits.effects.grain_amount === 0);

  function resetVignette(): void {
    editor.edits.effects.vignette_amount = NEUTRAL_EFFECTS.vignette_amount;
    editor.edits.effects.vignette_midpoint = NEUTRAL_EFFECTS.vignette_midpoint;
    editor.edits.effects.vignette_feather = NEUTRAL_EFFECTS.vignette_feather;
    editor.edits.effects.vignette_roundness = NEUTRAL_EFFECTS.vignette_roundness;
    editor.onCommit('Reset Vignette');
  }

  function resetGrain(): void {
    editor.edits.effects.grain_amount = NEUTRAL_EFFECTS.grain_amount;
    editor.edits.effects.grain_size = NEUTRAL_EFFECTS.grain_size;
    editor.edits.effects.grain_roughness = NEUTRAL_EFFECTS.grain_roughness;
    editor.onCommit('Reset Grain');
  }
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Vignette</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset Vignette"
        aria-label="Reset Vignette"
        onclick={resetVignette}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Amount"
      commitAction="Vignette Amount"
      bind:value={editor.edits.effects.vignette_amount}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Midpoint"
      commitAction="Vignette Midpoint"
      bind:value={editor.edits.effects.vignette_midpoint}
      min={0}
      max={100}
      step={1}
      defaultValue={50}
      disabled={vignetteInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Feather"
      commitAction="Vignette Feather"
      bind:value={editor.edits.effects.vignette_feather}
      min={0}
      max={100}
      step={1}
      defaultValue={50}
      disabled={vignetteInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Roundness"
      commitAction="Vignette Roundness"
      bind:value={editor.edits.effects.vignette_roundness}
      min={-100}
      max={100}
      step={1}
      disabled={vignetteInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
  <div class="flex flex-col gap-2.5 pt-3">
    <div class="flex items-center justify-between">
      <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Grain</div>
      <button
        type="button"
        class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
        title="Reset Grain"
        aria-label="Reset Grain"
        onclick={resetGrain}
      >
        <Icon path={mdiRestore} size={14} />
      </button>
    </div>
    <SliderRow
      label="Amount"
      commitAction="Grain Amount"
      bind:value={editor.edits.effects.grain_amount}
      min={0}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Size"
      commitAction="Grain Size"
      bind:value={editor.edits.effects.grain_size}
      min={0}
      max={100}
      step={1}
      defaultValue={25}
      disabled={grainInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Roughness"
      commitAction="Grain Roughness"
      bind:value={editor.edits.effects.grain_roughness}
      min={0}
      max={100}
      step={1}
      defaultValue={50}
      disabled={grainInactive}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
</div>
