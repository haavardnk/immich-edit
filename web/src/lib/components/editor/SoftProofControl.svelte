<script lang="ts">
  import ElementTooltip from '#lib/components/ElementTooltip.svelte';
  import CheckboxRow from '#lib/components/CheckboxRow.svelte';
  import { segmented } from '#lib/components/segmented.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { ColorSpaceOpt } from '#lib/api/export.ts';
  import { RadioGroup } from 'bits-ui';
  import { MediaQuery } from 'svelte/reactivity';
  const wideGamut = new MediaQuery('color-gamut: p3');
</script>

<div class="flex flex-col gap-1.5 px-3 pb-2">
  <span class="section-label pt-1.5 text-dark/65">Soft proof</span>
  <div class="flex items-center justify-between gap-3 text-xs">
    <span>Proof space</span>
    <RadioGroup.Root
      bind:value={() => editor.proofSpace, (v) => editor.setProofSpace(v as ColorSpaceOpt)}
      orientation="horizontal"
      aria-label="Proof space"
      class="{segmented.text.group} w-36"
    >
      <RadioGroup.Item value="srgb" class="{segmented.text.radio} flex-1">sRGB</RadioGroup.Item>
      <ElementTooltip text={wideGamut.current ? null : 'Needs a wide-gamut display'}>
        {#snippet child({ props })}
          <span {...props} class="flex flex-1">
            <RadioGroup.Item
              value="displayp3"
              disabled={!wideGamut.current}
              class="{segmented.text.radio} flex-1 data-disabled:opacity-40"
              >Display P3</RadioGroup.Item
            >
          </span>
        {/snippet}
      </ElementTooltip>
    </RadioGroup.Root>
  </div>
  <CheckboxRow
    label="Show gamut warning"
    class="text-xs"
    checked={editor.gamutWarn}
    onChange={editor.toggleGamutWarn}
  />
</div>
