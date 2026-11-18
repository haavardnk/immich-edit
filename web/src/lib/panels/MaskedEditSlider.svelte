<script lang="ts">
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { MaskParam } from '#lib/editor/params/masks.ts';

  let { layerId, param }: { layerId: string; param: MaskParam } = $props();

  const value = $derived(editor.edits.masks.find((l) => l.id === layerId)?.edits[param.key] ?? 0);

  function live(v: number): void {
    editor.setMaskLayerEdit(layerId, param.key, v);
  }

  function commit(): void {
    void editor.commitMasks();
  }
</script>

<SliderRow
  label={param.label}
  {value}
  range={param}
  onLive={live}
  onCommit={commit}
  onDragStart={editor.holdMaskOverlay}
  onDragEnd={editor.releaseMaskOverlay}
/>
