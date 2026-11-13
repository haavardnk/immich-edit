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
    zoom = 3,
    overlay = null,
    ring = null
  }: {
    source: PreviewSurface;
    u: number;
    v: number;
    width: number;
    zoom?: number;
    overlay?: HTMLCanvasElement | null;
    ring?: number | null;
  } = $props();

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
    if (!overlay || overlay.width <= 0 || overlay.height <= 0) return;
    const over = (MAGNIFIER_SIZE / zoom) * (overlay.width / width);
    context.drawImage(
      overlay,
      u * overlay.width - over / 2,
      v * overlay.height - over / 2,
      over,
      over,
      0,
      0,
      pixels,
      pixels
    );
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
  {#if ring}
    <span
      data-testid="touch-magnifier-ring"
      class="absolute top-1/2 left-1/2 -translate-1/2 rounded-full border-2 border-image-light shadow-[0_0_0_1px_var(--color-control-shadow)]"
      style:width="{Math.min(ring, MAGNIFIER_SIZE - 8)}px"
      style:height="{Math.min(ring, MAGNIFIER_SIZE - 8)}px"
    ></span>
  {:else}
    <span class="absolute top-1/2 left-1/2 size-3 -translate-1/2 border border-image-accent"></span>
  {/if}
</span>
