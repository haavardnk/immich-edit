<script lang="ts">
  import { compactSegmentedControlClass } from '#lib/components/editor/controls/segmentedControl.ts';
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { BRUSH_FLOW, BRUSH_HARDNESS, BRUSH_SIZE } from '#lib/editor/params/brush.ts';
  import { formatBrushSize } from '#lib/utils/brushSize.ts';
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
    range={BRUSH_SIZE}
    onLive={(v: number) => editor.setBrushTool({ size: v })}
    onCommit={() => editor.setBrushTool({ size })}
    format={(v: number) => formatBrushSize(v, editor.meta)}
  />
  <SliderRow
    label="Hardness"
    value={hardness}
    range={BRUSH_HARDNESS}
    onLive={(v: number) => editor.setBrushTool({ hardness: v })}
    onCommit={() => editor.setBrushTool({ hardness })}
  />
  <SliderRow
    label="Flow"
    value={flow}
    range={BRUSH_FLOW}
    onLive={(v: number) => editor.setBrushTool({ flow: v })}
    onCommit={() => editor.setBrushTool({ flow })}
  />
</div>
