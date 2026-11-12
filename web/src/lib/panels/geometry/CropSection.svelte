<script lang="ts">
  import SectionHeader from '#lib/components/editor/controls/SectionHeader.svelte';
  import { hint } from '#lib/shortcuts/labels.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { cropIsDefault, type GeometrySession } from '#lib/stores/editor/geometry.svelte.ts';
  import { CROP_GRIDS, ui } from '#lib/stores/ui.svelte.ts';
  import { IconButton } from '@immich/ui';
  import { mdiGrid, mdiRotate3dVariant } from '@mdi/js';
  import { TURNING_GRIDS } from '#lib/editor/cropGuides.ts';
  import RatioChips from './RatioChips.svelte';
  import RatioField from './RatioField.svelte';
  import { draftOutputSize } from '#lib/panels/geometry/ratioActions.ts';

  let { sess }: { sess: GeometrySession } = $props();

  const cropModified = $derived(!cropIsDefault(sess) || sess.draftAspect.kind !== 'original');
  const gridLabel = $derived(CROP_GRIDS.find((g) => g.id === ui.cropGrid)?.label ?? '');
  const outputSize = $derived(draftOutputSize());

  function resetCrop(): void {
    void editor.setGeometryAspect({ kind: 'original' }, 'Reset crop');
  }
</script>

<div class="flex flex-col gap-1.5 pb-2">
  <SectionHeader title="Crop" modified={cropModified} onReset={resetCrop}>
    {#snippet actions()}
      {#if outputSize}
        <span class="text-[11px] tabular-nums text-dark/55" data-testid="crop-output-size"
          >{outputSize.w} × {outputSize.h} px</span
        >
      {/if}
      <IconButton
        size="tiny"
        variant="ghost"
        color="secondary"
        icon={mdiGrid}
        title={hint(`Crop guide: ${gridLabel}`, 'cropGrid')}
        aria-label={`Crop guide: ${gridLabel}`}
        onclick={ui.cycleCropGrid}
      />
      {#if TURNING_GRIDS.has(ui.cropGrid)}
        <IconButton
          size="tiny"
          variant="ghost"
          color="secondary"
          icon={mdiRotate3dVariant}
          title={hint('Turn guide', 'cropGuideTurn')}
          aria-label="Turn guide"
          onclick={ui.turnCropGuide}
        />
      {/if}
    {/snippet}
  </SectionHeader>
  <span class="text-[10px] font-medium tracking-wider text-dark/55 uppercase select-none"
    >Aspect ratio</span
  >
  <RatioChips />
  <RatioField />
</div>
