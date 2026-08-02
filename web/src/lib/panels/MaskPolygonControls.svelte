<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskComponent } from '$lib/types/edits';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const polygon = $derived(component.kind.kind === 'polygon' ? component.kind : null);

  function updateFeather(value: number): void {
    if (!polygon) return;
    editor.updateMaskComponentKind(layerId, component.id, { ...polygon, feather: value }, true);
  }
</script>

{#if polygon}
  <div class="mt-2 flex flex-col gap-2.5">
    <SliderRow
      label="Feather"
      value={polygon.feather}
      min={0}
      max={0.5}
      step={0.005}
      defaultValue={0.05}
      onLive={updateFeather}
      onCommit={() => void editor.commitMasks()}
      format={(value: number) => value.toFixed(3)}
    />
    <p class="px-1 text-[10px] text-immich-dark-fg/40">
      Drag a corner to move it, the small dot between corners to add one, and double-click a corner
      to remove it.
    </p>
  </div>
{/if}
