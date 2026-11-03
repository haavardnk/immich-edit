<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import { POLYGON_FEATHER } from '#lib/editor/params/maskComponents.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { MaskComponent } from '#lib/types/edits.ts';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const polygon = $derived(component.kind.kind === 'polygon' ? component.kind : null);

  function updateFeather(value: number): void {
    if (!polygon) return;
    editor.updateMaskComponentKind(layerId, component.id, { ...polygon, feather: value }, true);
  }
</script>

{#if polygon}
  <div class="mt-1.5 flex flex-col gap-1.5">
    <SliderRow
      label="Feather"
      value={polygon.feather}
      min={POLYGON_FEATHER.min}
      max={POLYGON_FEATHER.max}
      step={POLYGON_FEATHER.step}
      defaultValue={POLYGON_FEATHER.defaultValue}
      onLive={updateFeather}
      onCommit={() => void editor.commitMasks()}
      format={POLYGON_FEATHER.format}
    />
  </div>
{/if}
