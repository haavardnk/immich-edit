<script lang="ts">
  import ParamSlider from '#lib/components/editor/controls/ParamSlider.svelte';
  import ResetButton from '#lib/components/editor/controls/ResetButton.svelte';
  import {
    commitParamReset,
    commitSectionReset
  } from '#lib/components/editor/controls/paramReset.ts';
  import { segmented } from '#lib/components/segmented.ts';
  import { HSL, HSL_BAND_PARAMS } from '#lib/editor/params/hsl.ts';
  import { paramsModified } from '#lib/editor/params/define.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { keyLabel } from '#lib/shortcuts/chord.ts';
  import { HSL_BAND_NAMES, HSL_BAND_COLORS } from '#lib/types/edits.ts';
  import { Tooltip } from '@immich/ui';
  import { RadioGroup } from 'bits-ui';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { settingFilter, shownGroup } from '#lib/components/editor/controls/settingFilter.ts';
  import { settingSearch } from '#lib/stores/settingSearch.svelte.ts';

  const filter = settingFilter();
  let activeBand = $state(0);

  $effect(() => {
    const revealed = settingSearch.revealedIn(HSL_BAND_PARAMS);
    if (revealed >= 0) activeBand = revealed;
  });

  const band = $derived(shownGroup(filter, HSL_BAND_PARAMS, activeBand));
  const bandName = $derived(HSL_BAND_NAMES[band] ?? '');
  const bandParams = $derived(HSL_BAND_PARAMS[band] ?? []);
  const modifiedBands = $derived(
    HSL_BAND_PARAMS.map((params) => paramsModified(editor.edits, params, editor.isRaw))
  );
</script>

<div class="flex flex-col gap-1">
  <RadioGroup.Root
    bind:value={() => String(band), (v) => (activeBand = Number(v))}
    orientation="horizontal"
    aria-label="Colour band"
    class="grid grid-cols-8 gap-0.5 touch:py-1"
  >
    {#each HSL_BAND_NAMES as name, i (name)}
      <Tooltip text={name}>
        {#snippet child({ props })}
          <RadioGroup.Item
            value={String(i)}
            class="{segmented.tool
              .swatch} relative touch:mx-auto touch:size-8 touch:rounded-full touch:ring-0 touch:before:absolute touch:before:-inset-1.5 touch:aria-checked:ring-2 touch:aria-checked:ring-light-950 touch:aria-checked:ring-offset-2 touch:aria-checked:ring-offset-editor-panel"
            style="background-color: {HSL_BAND_COLORS[i]}"
            aria-label={name}
            {...props}
          >
            {#if modifiedBands[i]}
              <span
                data-modified-indicator
                aria-hidden="true"
                class="absolute -right-0.5 -bottom-0.5 size-2 rounded-full bg-primary ring-2 ring-editor-panel"
              ></span>
            {/if}
          </RadioGroup.Item>
        {/snippet}
      </Tooltip>
    {/each}
  </RadioGroup.Root>
  <div class="flex h-6 items-center justify-between">
    <div class="text-[11px] font-medium tracking-wide text-dark/65 uppercase touch:text-[12px]">
      {bandName}
    </div>
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
