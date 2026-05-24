<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';

  const vignetteInactive = $derived(editor.edits.effects.vignette_amount === 0);
  const grainInactive = $derived(editor.edits.effects.grain_amount === 0);
</script>

<div class="flex flex-col divide-y divide-white/5">
  <div class="flex flex-col gap-2.5 pb-3">
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Vignette</div>
    <SliderRow
      label="Amount"
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
    <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40">Grain</div>
    <SliderRow
      label="Amount"
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
