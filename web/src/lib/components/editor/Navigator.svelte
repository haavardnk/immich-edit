<script lang="ts">
  import { grabOffset, panTo, visibleBox } from '#lib/editor/navigator.ts';
  import { pointerDrag } from '#lib/gestures/drag.ts';
  import { clampedPoint, type Point } from '#lib/gestures/point.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { View, ViewportMetrics } from '#lib/utils/imageViewport.ts';

  let {
    metrics,
    view,
    onpan,
    class: className = ''
  }: {
    metrics: ViewportMetrics;
    view: View;
    onpan: (view: View) => void;
    class?: string;
  } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let dragging = $state(false);

  const box = $derived(visibleBox(metrics, view));
  const ratio = $derived(metrics.natural.w / metrics.natural.h);

  $effect(() => {
    const target = canvas;
    const frame = editor.previewFrame;
    if (!target || !frame) return;
    target.width = target.clientWidth * 2;
    target.height = target.clientHeight * 2;
    target
      .getContext('2d', { colorSpace: frame.colorSpace })
      ?.drawImage(frame.bitmap, 0, 0, target.width, target.height);
  });

  function pan(e: PointerEvent, grab: Point): void {
    onpan(panTo(metrics, view, clampedPoint(e, e.currentTarget as Element), grab));
  }

  const drag = pointerDrag<Point>({
    start(e) {
      if (e.button !== 0) return null;
      e.preventDefault();
      dragging = true;
      const grab = grabOffset(box, clampedPoint(e, e.currentTarget as Element));
      pan(e, grab);
      return grab;
    },
    move: pan,
    end() {
      dragging = false;
    }
  });

  const handlers = drag.on();

  function onpointerdown(e: PointerEvent): void {
    e.stopPropagation();
    handlers.onpointerdown(e);
  }
</script>

<div
  data-testid="navigator"
  aria-hidden="true"
  class="absolute z-10 touch-none overflow-hidden rounded-lg shadow-lg ring-1 ring-white/14 [--nav-h:120px] [--nav-w:180px] touch:[--nav-h:80px] touch:[--nav-w:120px] {dragging
    ? 'cursor-grabbing'
    : 'cursor-grab'} {className}"
  style:width="min(var(--nav-w), calc(var(--nav-h) * {ratio}))"
  style:aspect-ratio={ratio}
  {...handlers}
  {onpointerdown}
>
  {#if editor.previewFrame}
    <canvas bind:this={canvas} class="block size-full"></canvas>
  {:else if editor.previewUrl}
    <img
      src={editor.previewUrl}
      alt=""
      draggable="false"
      class="block size-full select-none"
      style="image-orientation: none;"
    />
  {/if}
  <span
    data-testid="navigator-box"
    class="pointer-events-none absolute border-2 border-white shadow-[0_0_0_999px_rgb(0_0_0/0.45)]"
    style:left="{box[0] * 100}%"
    style:top="{box[1] * 100}%"
    style:width="{box[2] * 100}%"
    style:height="{box[3] * 100}%"
  ></span>
</div>
