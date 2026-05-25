<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import Icon from '$lib/components/Icon.svelte';
  import { mdiRestore } from '@mdi/js';
  import { editor } from '$lib/stores/editor.svelte';
  import { HSL_BAND_NAMES, HSL_BAND_COLORS, HSL_BAND_HUES } from '$lib/types/edits';

  let activeBand = $state(0);

  function resetBand(i: number): void {
    const b = editor.edits.color.hsl.bands[i];
    b.hue = 0;
    b.sat = 0;
    b.lum = 0;
    editor.onCommit();
  }

  function resetAllHsl(): void {
    for (const b of editor.edits.color.hsl.bands) {
      b.hue = 0;
      b.sat = 0;
      b.lum = 0;
    }
    editor.onCommit();
  }

  const bandHue = $derived(HSL_BAND_HUES[activeBand]);
  const currentBand = $derived(editor.edits.color.hsl.bands[activeBand]);
  const effectiveHue = $derived(((bandHue + currentBand.hue + 360) % 360));
  const effectiveSat = $derived(Math.max(0, Math.min(100, (currentBand.sat + 100) / 2)));
  const hueGradient = $derived(
    `linear-gradient(to right, hsl(${(bandHue - 100 + 360) % 360}, 50%, 50%), hsl(${bandHue}, 50%, 50%), hsl(${(bandHue + 100) % 360}, 50%, 50%))`
  );
  const satGradient = $derived(
    `linear-gradient(to right, hsl(${effectiveHue}, 0%, 50%), hsl(${effectiveHue}, 100%, 50%))`
  );
  const lumGradient = $derived(
    `linear-gradient(to right, hsl(${effectiveHue}, ${effectiveSat}%, 0%), hsl(${effectiveHue}, ${effectiveSat}%, 50%), hsl(${effectiveHue}, ${effectiveSat}%, 100%))`
  );
</script>

<div class="flex flex-col gap-2.5">
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
  <div class="flex items-center justify-between px-1">
    <div class="text-[11px] text-immich-dark-fg/70">{HSL_BAND_NAMES[activeBand]}</div>
    <button
      type="button"
      class="text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors"
      title="Reset {HSL_BAND_NAMES[activeBand]}  —  shift-click to reset all bands"
      aria-label="Reset {HSL_BAND_NAMES[activeBand]}"
      onclick={(e) => (e.shiftKey ? resetAllHsl() : resetBand(activeBand))}
    >
      <Icon path={mdiRestore} size={14} />
    </button>
  </div>
  <SliderRow
    label="Hue"
    bind:value={editor.edits.color.hsl.bands[activeBand].hue}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
    gradient={hueGradient}
  />
  <SliderRow
    label="Saturation"
    bind:value={editor.edits.color.hsl.bands[activeBand].sat}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
    gradient={satGradient}
  />
  <SliderRow
    label="Luminance"
    bind:value={editor.edits.color.hsl.bands[activeBand].lum}
    min={-100}
    max={100}
    step={1}
    onLive={editor.onLive}
    onCommit={editor.onCommit}
    format={(v: number) => v.toFixed(0)}
    gradient={lumGradient}
  />
</div>
