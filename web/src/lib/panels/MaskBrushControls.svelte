<script lang="ts">
  import { compactSegmentedControlClass } from '$lib/components/editor/controls/segmentedControl';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { Button } from '@immich/ui';

  const size = $derived(editor.brushTool.size);
  const hardness = $derived(editor.brushTool.hardness);
  const flow = $derived(editor.brushTool.flow);
</script>

<div class="mt-1 flex flex-col gap-1">
  <div class="flex h-7 items-center justify-between">
    <div class="text-[10px] font-medium text-dark/65">Brush</div>
    <div class="{compactSegmentedControlClass} flex text-[10px]">
      <Button
        type="button"
        size="tiny"
        variant={editor.brushTool.mode === 'paint' ? 'filled' : 'ghost'}
        color={editor.brushTool.mode === 'paint' ? 'primary' : 'secondary'}
        onclick={() => editor.setBrushTool({ mode: 'paint' })}
      >
        Paint
      </Button>
      <Button
        type="button"
        size="tiny"
        variant={editor.brushTool.mode === 'erase' ? 'filled' : 'ghost'}
        color={editor.brushTool.mode === 'erase' ? 'primary' : 'secondary'}
        onclick={() => editor.setBrushTool({ mode: 'erase' })}
      >
        Erase
      </Button>
    </div>
  </div>
  <SliderRow
    label="Size"
    value={size}
    min={0.005}
    max={0.5}
    step={0.005}
    defaultValue={0.08}
    onLive={(v: number) => editor.setBrushTool({ size: v })}
    onCommit={() => editor.setBrushTool({ size })}
    format={(v: number) => v.toFixed(3)}
  />
  <SliderRow
    label="Hardness"
    value={hardness}
    min={0}
    max={1}
    step={0.01}
    defaultValue={0.5}
    onLive={(v: number) => editor.setBrushTool({ hardness: v })}
    onCommit={() => editor.setBrushTool({ hardness })}
    format={(v: number) => v.toFixed(2)}
  />
  <SliderRow
    label="Flow"
    value={flow}
    min={0.01}
    max={1}
    step={0.01}
    defaultValue={0.8}
    onLive={(v: number) => editor.setBrushTool({ flow: v })}
    onCommit={() => editor.setBrushTool({ flow })}
    format={(v: number) => v.toFixed(2)}
  />
</div>
