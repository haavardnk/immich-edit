<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { HSL_BAND_NAMES, HSL_BAND_COLORS } from '$lib/types/edits';

  let activeBand = $state(0);
</script>

<div class="flex flex-col gap-2.5">
  <SliderRow
    label="Saturation"
    bind:value={editor.edits.basic.saturation}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />
  <SliderRow
    label="Vibrance"
    bind:value={editor.edits.basic.vibrance}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
  />

  <div class="flex flex-col gap-2 pt-2 mt-1 border-t border-white/10">
    <div class="text-[10px] uppercase tracking-wide text-immich-dark-fg/60 px-1">HSL</div>
    <div class="grid grid-cols-8 gap-1">
      {#each HSL_BAND_NAMES as name, i (name)}
        <button
          type="button"
          class="h-6 rounded ring-1 ring-white/10 hover:ring-white/40 transition-shadow {activeBand === i ? 'ring-2 ring-white/80' : ''}"
          style="background-color: {HSL_BAND_COLORS[i]}"
          title={name}
          onclick={() => (activeBand = i)}
        ></button>
      {/each}
    </div>
    <div class="text-[11px] text-immich-dark-fg/70 px-1">{HSL_BAND_NAMES[activeBand]}</div>
    <SliderRow
      label="Hue"
      bind:value={editor.edits.color.hsl.bands[activeBand].hue}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Sat"
      bind:value={editor.edits.color.hsl.bands[activeBand].sat}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
    <SliderRow
      label="Lum"
      bind:value={editor.edits.color.hsl.bands[activeBand].lum}
      min={-100}
      max={100}
      step={1}
      onLive={editor.onLive}
      onCommit={editor.onCommit}
      format={(v: number) => v.toFixed(0)}
    />
  </div>
</div>
