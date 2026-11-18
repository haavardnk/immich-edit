<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import ResetButton from '#lib/components/editor/controls/ResetButton.svelte';
  import {
    commitParamReset,
    commitSectionReset
  } from '#lib/components/editor/controls/paramReset.ts';
  import { compactSegmentedSwatchItemClass } from '#lib/components/editor/controls/segmentedControl.ts';
  import { HSL, HSL_BAND_PARAMS } from '#lib/editor/params/hsl.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { HSL_BAND_NAMES, HSL_BAND_COLORS } from '#lib/types/edits.ts';
  import { Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import { held } from '#lib/input/modifiers.svelte.ts';

  let activeBand = $state(0);

  const bandName = $derived(HSL_BAND_NAMES[activeBand] ?? '');
  const bandParams = $derived(HSL_BAND_PARAMS[activeBand] ?? []);
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
    <div class="text-[11px] text-dark/65">{bandName}</div>
    <ResetButton
      title="Reset {bandName}  —  {keyLabel('Shift')}-click to reset all bands"
      label="Reset {bandName}"
      onclick={(e) =>
        held('resetAll', e)
          ? commitSectionReset(HSL)
          : commitParamReset(bandParams, `Reset ${bandName}`)}
    />
  </div>
  {#each bandParams as param (param.id)}
    <ParamSlider {param} />
  {/each}
</div>
