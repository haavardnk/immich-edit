<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskedEditKey } from '$lib/types/edits';

  let {
    layerId,
    eKey,
    label,
    min,
    max,
    step = 1,
    format,
    gradient
  }: {
    layerId: string;
    eKey: MaskedEditKey;
    label: string;
    min: number;
    max: number;
    step?: number;
    format?: (v: number) => string;
    gradient?: string;
  } = $props();

  const value = $derived(editor.edits.masks.find((l) => l.id === layerId)?.edits[eKey] ?? 0);

  function live(v: number): void {
    editor.setMaskLayerEdit(layerId, eKey, v);
  }

  function commit(): void {
    void editor.commitMasks();
  }
</script>

<SliderRow
  {label}
  {value}
  {min}
  {max}
  {step}
  onLive={live}
  onCommit={commit}
  format={format ?? ((v: number) => v.toFixed(0))}
  {gradient}
/>
