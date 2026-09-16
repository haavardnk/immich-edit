<script lang="ts">
  import type { Raster } from './rasterize';

  let { raster }: { raster: Raster | null } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);

  $effect(() => {
    const el = canvas;
    const source = raster;
    if (!el || !source) return;
    const ctx = el.getContext('2d');
    if (!ctx) return;
    el.width = source.width;
    el.height = source.height;
    ctx.putImageData(new ImageData(source.data, source.width, source.height), 0, 0);
  });
</script>

<canvas bind:this={canvas} class="block h-full w-full"></canvas>
