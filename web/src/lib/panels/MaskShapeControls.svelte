<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import { MASK_FEATHER } from '#lib/editor/params/maskComponents.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskModels } from '#lib/stores/maskModels.svelte.ts';
  import type { MaskComponent } from '#lib/types/edits.ts';
  import MaskBrushControls from './MaskBrushControls.svelte';
  import MaskClickTool from './MaskClickTool.svelte';
  import MaskGeneratedControls from './MaskGeneratedControls.svelte';
  import MaskPolygonControls from './MaskPolygonControls.svelte';
  import MaskRangeControls from './MaskRangeControls.svelte';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const kind = $derived(component.kind);
  const clickPoints = $derived(
    component.generated?.kind === 'click' ? (component.generated.points ?? []) : []
  );

  function onFeatherLive(v: number): void {
    editor.setMaskComponentFeather(layerId, component.id, v);
  }
</script>

{#if kind.kind === 'linear' || kind.kind === 'radial'}
  <div class="mt-1 flex flex-col gap-1">
    <SliderRow
      label="Feather"
      value={kind.feather}
      range={MASK_FEATHER}
      onLive={onFeatherLive}
      onCommit={() => void editor.commitMasks()}
    />
  </div>
{/if}

{#if kind.kind === 'brush' && maskModels.installed('click')}
  <MaskClickTool {layerId} points={clickPoints} />
{/if}

{#if component.generated}
  <MaskGeneratedControls {layerId} {component} />
{:else if kind.kind === 'brush'}
  <MaskBrushControls />
{/if}

{#if kind.kind === 'luma_range' || kind.kind === 'color_range'}
  <MaskRangeControls {layerId} {component} />
{/if}

{#if kind.kind === 'polygon'}
  <MaskPolygonControls {layerId} {component} />
{/if}
