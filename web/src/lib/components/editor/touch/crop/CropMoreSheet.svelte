<script lang="ts">
  import MenuRow from '#lib/components/MenuRow.svelte';
  import ActionSheet from '#lib/components/sheet/ActionSheet.svelte';
  import { TURNING_GRIDS } from '#lib/editor/cropGuides.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { CROP_GRIDS, ui } from '#lib/stores/ui.svelte.ts';
  import {
    mdiCropFree,
    mdiFlipVertical,
    mdiGrid,
    mdiRotate3dVariant,
    mdiRotateRight
  } from '@mdi/js';

  let { open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void } = $props();

  const flipV = $derived(editor.geometrySession?.draftFlipV ?? editor.edits.geometry.flip_v);
  const gridLabel = $derived(CROP_GRIDS.find((g) => g.id === ui.cropGrid)?.label ?? '');
</script>

<ActionSheet {open} {onOpenChange} title="More crop actions">
  <MenuRow
    icon={mdiRotateRight}
    label="Rotate right 90°"
    data-menu-stay
    onclick={() => editor.rotateStep(90)}
  />
  <MenuRow
    icon={mdiFlipVertical}
    label="Flip vertical"
    pressed={flipV}
    data-menu-stay
    onclick={() => editor.flipStep('v')}
  />
  <MenuRow icon={mdiGrid} label="Crop guide" data-menu-stay onclick={ui.cycleCropGrid}>
    <span class="shrink-0 text-[13px] text-dark/65">{gridLabel}</span>
  </MenuRow>
  {#if TURNING_GRIDS.has(ui.cropGrid)}
    <MenuRow
      icon={mdiRotate3dVariant}
      label="Turn guide"
      data-menu-stay
      onclick={ui.turnCropGuide}
    />
  {/if}
  <MenuRow
    icon={mdiCropFree}
    label="Reset crop"
    onclick={() => void editor.setGeometryAspect({ kind: 'original' }, 'Reset crop')}
  />
</ActionSheet>
