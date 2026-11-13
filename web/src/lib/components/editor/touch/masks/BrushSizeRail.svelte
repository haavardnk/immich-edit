<script lang="ts">
  import { BRUSH_SIZE } from '#lib/editor/params/brush.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { paintInput } from '#lib/stores/editor/paintInput.svelte.ts';
  import { formatBrushSize } from '#lib/utils/brushSize.ts';
  import { Slider } from 'bits-ui';

  const size = $derived(editor.brushTool.size);
  const readout = $derived(formatBrushSize(size, editor.meta).replace(' px', ''));

  function live(value: number): void {
    paintInput.sizing = true;
    editor.setBrushTool({ size: value });
  }
</script>

<div
  data-testid="brush-size-rail"
  class="absolute top-1/2 left-3 z-20 flex h-52 w-11 -translate-y-1/2 flex-col items-center gap-2 rounded-full bg-black/55 py-3 text-white shadow-lg backdrop-blur-sm [[data-arrangement=side]_&]:top-[calc(50%+1.5rem)]"
>
  <span class="text-[10px] font-medium text-white/70" aria-hidden="true">Size</span>
  <Slider.Root
    type="single"
    orientation="vertical"
    min={BRUSH_SIZE.min}
    max={BRUSH_SIZE.max}
    step={BRUSH_SIZE.step}
    value={size}
    onValueChange={live}
    onValueCommit={() => (paintInput.sizing = false)}
    class="relative flex w-full flex-1 touch-none justify-center"
  >
    <span class="relative h-full w-1 rounded-full bg-white/25">
      <Slider.Range class="absolute w-full rounded-full bg-primary" />
    </span>
    <Slider.Thumb
      index={0}
      aria-label="Brush size"
      aria-valuetext={formatBrushSize(size, editor.meta)}
      class="block size-6 rounded-full bg-white shadow-md outline-none before:absolute before:-inset-2.5 focus-visible:ring-2 focus-visible:ring-primary"
    />
  </Slider.Root>
  <span class="font-mono text-[11px] tabular-nums">{readout}</span>
</div>
