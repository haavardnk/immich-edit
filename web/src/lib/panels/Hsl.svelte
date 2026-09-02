<script lang="ts">
  import EditSlider from '$lib/components/editor/controls/EditSlider.svelte';
  import ResetButton from '$lib/components/editor/controls/ResetButton.svelte';
  import { compactSegmentedSwatchItemClass } from '$lib/components/editor/controls/segmentedControl';
  import { editor } from '$lib/stores/editor.svelte';
  import { keyLabel } from '$lib/keybinds';
  import { HSL_BAND_NAMES, HSL_BAND_COLORS, HSL_BAND_HUES } from '$lib/types/edits';
  import { Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';

  let activeBand = $state(0);

  function resetBand(i: number): void {
    const b = editor.edits.color.hsl.bands[i];
    b.hue = 0;
    b.sat = 0;
    b.lum = 0;
    editor.onCommit(`Reset ${HSL_BAND_NAMES[i]}`);
  }

  function resetAllHsl(): void {
    for (const b of editor.edits.color.hsl.bands) {
      b.hue = 0;
      b.sat = 0;
      b.lum = 0;
    }
    editor.onCommit('Reset HSL');
  }

  const bandHue = $derived(HSL_BAND_HUES[activeBand]);
  const currentBand = $derived(editor.edits.color.hsl.bands[activeBand]);
  const effectiveHue = $derived((bandHue + currentBand.hue + 360) % 360);
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

<div class="flex flex-col gap-1">
  <RadioGroup.Root
    bind:value={() => String(activeBand), (v) => (activeBand = Number(v))}
    orientation="horizontal"
    aria-label="Colour band"
    class="grid grid-cols-8 gap-0.5"
  >
    {#each HSL_BAND_NAMES as name, i (name)}
      <Tooltip text={name}>
        {#snippet child({ props })}
          <RadioGroup.Item
            value={String(i)}
            class={compactSegmentedSwatchItemClass}
            style="background-color: {HSL_BAND_COLORS[i]}"
            aria-label={name}
            {...props}
          />
        {/snippet}
      </Tooltip>
    {/each}
  </RadioGroup.Root>
  <div class="flex h-6 items-center justify-between">
    <div class="text-[11px] text-dark/65">{HSL_BAND_NAMES[activeBand]}</div>
    <ResetButton
      title="Reset {HSL_BAND_NAMES[activeBand]}  —  {keyLabel('Shift')}-click to reset all bands"
      label="Reset {HSL_BAND_NAMES[activeBand]}"
      onclick={(e) => (e.shiftKey ? resetAllHsl() : resetBand(activeBand))}
    />
  </div>
  <EditSlider
    label="Hue"
    commitAction={`${HSL_BAND_NAMES[activeBand]} Hue`}
    bind:value={editor.edits.color.hsl.bands[activeBand].hue}
    min={-100}
    max={100}
    gradient={hueGradient}
  />
  <EditSlider
    label="Saturation"
    commitAction={`${HSL_BAND_NAMES[activeBand]} Saturation`}
    bind:value={editor.edits.color.hsl.bands[activeBand].sat}
    min={-100}
    max={100}
    gradient={satGradient}
  />
  <EditSlider
    label="Luminance"
    commitAction={`${HSL_BAND_NAMES[activeBand]} Luminance`}
    bind:value={editor.edits.color.hsl.bands[activeBand].lum}
    min={-100}
    max={100}
    gradient={lumGradient}
  />
</div>
