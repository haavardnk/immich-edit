<script lang="ts">
  import type { CropRatio } from '#lib/api/preferences.ts';
  import MenuRow from '#lib/components/MenuRow.svelte';
  import SegmentedChoice from '#lib/components/settings/SegmentedChoice.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { preferences } from '#lib/stores/preferences.svelte.ts';
  import {
    PRESET_RATIOS,
    ratioKey,
    ratioLabel,
    sourceRatioLabel
  } from '#lib/panels/geometry/aspects.ts';
  import {
    canFlipOrientation,
    draftPortrait,
    isPicked,
    pickKind,
    pickRatio,
    setOrientation
  } from '#lib/panels/geometry/ratioActions.ts';
  import { cropReadout } from '#lib/panels/geometry/cropReadout.ts';
  import { Icon } from '@immich/ui';
  import {
    mdiCheck,
    mdiCropFree,
    mdiCropLandscape,
    mdiCropPortrait,
    mdiCropSquare,
    mdiFormTextbox,
    mdiImageOutline
  } from '@mdi/js';

  let {
    open,
    onOpenChange,
    oncustom
  }: { open: boolean; onOpenChange: (open: boolean) => void; oncustom: () => void } = $props();

  const portrait = $derived(draftPortrait());
  const readout = $derived(cropReadout());
  const asShot = $derived(
    editor.meta ? `as shot · ${sourceRatioLabel(editor.meta.source_w, editor.meta.source_h)}` : ''
  );
  const ratios = $derived([
    ...PRESET_RATIOS,
    ...preferences.savedRatios.filter(
      (saved) => !PRESET_RATIOS.some((preset) => ratioKey(preset) === ratioKey(saved))
    )
  ]);

  function ratioIcon(ratio: CropRatio): string {
    if (ratio.num === ratio.den) return mdiCropSquare;
    return portrait ? mdiCropPortrait : mdiCropLandscape;
  }
</script>

{#snippet picked(on: boolean)}
  {#if on}
    <Icon icon={mdiCheck} size="20px" class="shrink-0 text-primary" aria-hidden="true" />
  {/if}
{/snippet}

<ActionSheet {open} {onOpenChange} title="Aspect ratio" caption>
  {#if readout}
    <p class="-mt-1 pb-2 text-center text-[12px] text-dark/65 tabular-nums">
      {readout.label} at {readout.lead}
    </p>
  {/if}
  <div class="border-b border-hairline px-3 pb-3">
    <SegmentedChoice
      appearance="view"
      fill
      label="Orientation"
      value={portrait ? 'portrait' : 'landscape'}
      options={[
        {
          value: 'landscape',
          label: 'Landscape',
          icon: mdiCropLandscape,
          disabled: !canFlipOrientation()
        },
        {
          value: 'portrait',
          label: 'Portrait',
          icon: mdiCropPortrait,
          disabled: !canFlipOrientation()
        }
      ]}
      onChange={(next) => setOrientation(next === 'portrait')}
    />
  </div>
  <MenuRow
    icon={mdiImageOutline}
    label="Original"
    pressed={isPicked('original')}
    onclick={() => pickKind('original')}
  >
    <span class="shrink-0 text-[13px] text-dark/65">{asShot}</span>
    {@render picked(isPicked('original'))}
  </MenuRow>
  <MenuRow
    icon={mdiCropFree}
    label="Free"
    pressed={isPicked('free')}
    onclick={() => pickKind('free')}
  >
    {@render picked(isPicked('free'))}
  </MenuRow>
  {#each ratios as ratio (ratioKey(ratio))}
    <MenuRow
      icon={ratioIcon(ratio)}
      label={ratioLabel(ratio, portrait)}
      pressed={isPicked(ratio)}
      onclick={() => pickRatio(ratio)}
    >
      {@render picked(isPicked(ratio))}
    </MenuRow>
  {/each}
  <MenuRow icon={mdiFormTextbox} label="Custom…" onclick={oncustom}>
    <span class="shrink-0 text-[13px] text-dark/65">width : height</span>
  </MenuRow>
</ActionSheet>
