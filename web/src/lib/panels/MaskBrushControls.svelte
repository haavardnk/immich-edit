<script lang="ts">
  import { segmented } from '#lib/components/segmented.ts';
  import SliderRow from '#lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { BRUSH_FLOW, BRUSH_HARDNESS, BRUSH_SIZE } from '#lib/editor/params/brush.ts';
  import { formatBrushSize } from '#lib/utils/brushSize.ts';
  import { Button } from '@immich/ui';
  import { keysFor } from '#lib/shortcuts/labels.ts';
  import { brushErase } from '#lib/stores/editor/brushErase.svelte.ts';
  import { layout } from '#lib/device/layout.svelte.ts';
  import { mdiBrush, mdiEraser } from '@mdi/js';

  const TOUCH_SEGMENT = 'touch:h-full touch:flex-1 touch:rounded-[10px] touch:text-[13px]';

  let { showSize = true }: { showSize?: boolean } = $props();

  const size = $derived(editor.brushTool.size);
  const hardness = $derived(editor.brushTool.hardness);
  const flow = $derived(editor.brushTool.flow);
  const heldErase = $derived(brushErase.held && editor.brushTool.mode === 'paint');
</script>

<div class="mt-1 flex flex-col gap-1">
  <div class="flex h-7 items-center gap-2 touch:h-auto">
    <div class="me-auto text-[10px] font-medium text-dark/65 touch:hidden">Brush</div>
    {#if heldErase}
      <kbd class="rounded border border-hairline px-1.5 py-0.5 font-mono text-[10px] text-dark/65">
        {keysFor('brushEraseHold')}
      </kbd>
    {/if}
    <div
      class="{segmented.tool
        .group} flex text-[10px] touch:h-12 touch:flex-1 touch:gap-0.5 touch:rounded-xl touch:bg-ghost touch:p-0.5 touch:ring-0"
    >
      <Button
        type="button"
        size="tiny"
        variant={editor.brushTool.mode === 'paint' ? 'filled' : 'ghost'}
        color={editor.brushTool.mode === 'paint' ? 'primary' : 'secondary'}
        class={TOUCH_SEGMENT}
        leadingIcon={layout.mode === 'touch' ? mdiBrush : undefined}
        onclick={() => editor.setBrushTool({ mode: 'paint' })}
      >
        Paint
      </Button>
      <Button
        type="button"
        size="tiny"
        variant={editor.brushTool.mode === 'erase' ? 'filled' : 'ghost'}
        color={editor.brushTool.mode === 'erase' ? 'primary' : 'secondary'}
        class="{TOUCH_SEGMENT} {heldErase
          ? 'outline-1 -outline-offset-1 outline-primary outline-dashed'
          : ''}"
        leadingIcon={layout.mode === 'touch' ? mdiEraser : undefined}
        data-held={heldErase || undefined}
        onclick={() => editor.setBrushTool({ mode: 'erase' })}
      >
        Erase
      </Button>
    </div>
  </div>
  {#if showSize}
    <SliderRow
      label="Size"
      value={size}
      range={BRUSH_SIZE}
      onLive={(v: number) => editor.setBrushTool({ size: v })}
      onCommit={() => editor.setBrushTool({ size })}
      format={(v: number) => formatBrushSize(v, editor.meta)}
    />
  {/if}
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
