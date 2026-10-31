<script lang="ts">
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import {
    segmentedControlClass,
    segmentedRadioItemClass
  } from '#lib/components/editor/controls/segmentedControl.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { ColorSpaceOpt } from '#lib/api/export.ts';
  import { RadioGroup } from 'bits-ui';
  import { MediaQuery } from 'svelte/reactivity';
  const wideGamut = new MediaQuery('color-gamut: p3');
</script>

<div class="flex flex-col gap-1.5 px-3 pb-2">
  <span class="pt-1.5 text-[10px] tracking-wider text-dark/65 uppercase">Soft proof</span>
  <div class="flex items-center justify-between gap-3 text-xs">
    <span>Proof space</span>
    <RadioGroup.Root
      bind:value={() => editor.proofSpace, (v) => editor.setProofSpace(v as ColorSpaceOpt)}
      orientation="horizontal"
      aria-label="Proof space"
      class="{segmentedControlClass} w-36"
    >
      <RadioGroup.Item value="srgb" class="{segmentedRadioItemClass} flex-1">sRGB</RadioGroup.Item>
      <RadioGroup.Item
        value="displayp3"
        disabled={!wideGamut.current}
        title={wideGamut.current ? undefined : 'Needs a wide-gamut display'}
        class="{segmentedRadioItemClass} flex-1 data-disabled:opacity-40"
        >Display P3</RadioGroup.Item
      >
    </RadioGroup.Root>
  </div>
  <CheckboxRow
    label="Show gamut warning"
    class="text-xs"
    checked={editor.gamutWarn}
    onChange={editor.toggleGamutWarn}
  />
</div>
