<script lang="ts" module>
  export const MAGNIFIER_SIZE = 92;
  export const MAGNIFIER_LIFT = 64;
</script>

<script lang="ts">
  import { surfaceSize, type PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { devicePixelRatio } from 'svelte/reactivity/window';

  let {
    source,
    u,
    v,
    width,
    zoom = 3
  }: { source: PreviewSurface; u: number; v: number; width: number; zoom?: number } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);

  $effect(() => {
    const context = canvas?.getContext('2d');
    if (!canvas || !context) return;
    const { w, h } = surfaceSize(source);
    if (w <= 0 || h <= 0 || width <= 0) return;
    const pixels = Math.round(MAGNIFIER_SIZE * (devicePixelRatio.current || 1));
    canvas.width = pixels;
    canvas.height = pixels;
    const side = MAGNIFIER_SIZE / zoom / (width / w);
    context.imageSmoothingEnabled = false;
    context.clearRect(0, 0, pixels, pixels);
    context.drawImage(source, u * w - side / 2, v * h - side / 2, side, side, 0, 0, pixels, pixels);
  });
</script>

<span
  data-testid="touch-magnifier"
  class="pointer-events-none absolute z-10 block overflow-hidden rounded-full shadow-lg ring-2 ring-image-light"
  style:left="{u * 100}%"
  style:top="{v * 100}%"
  style:width="{MAGNIFIER_SIZE}px"
  style:height="{MAGNIFIER_SIZE}px"
  style:transform="translate(-50%, calc(-100% - {MAGNIFIER_LIFT}px))"
>
  <canvas bind:this={canvas} class="size-full"></canvas>
  <span class="absolute top-1/2 left-1/2 size-3 -translate-1/2 border border-image-accent"></span>
</span>
