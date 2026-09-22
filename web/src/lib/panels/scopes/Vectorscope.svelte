<script lang="ts">
  import ScopeCanvas from './ScopeCanvas.svelte';
  import { rasterizeVector } from './rasterize';
  import type { ScopeGrid } from '$lib/types/preview';

  let { grid, gain, zoom, size }: { grid: ScopeGrid; gain: number; zoom: number; size: number } =
    $props();

  const raster = $derived(rasterizeVector(grid, gain));

  const SKIN_ANGLE = 123;
  const BOX = 2.5;

  const targets = [
    { label: 'R', rgb: [0.75, 0, 0] },
    { label: 'Yl', rgb: [0.75, 0.75, 0] },
    { label: 'G', rgb: [0, 0.75, 0] },
    { label: 'Cy', rgb: [0, 0.75, 0.75] },
    { label: 'B', rgb: [0, 0, 0.75] },
    { label: 'Mg', rgb: [0.75, 0, 0.75] }
  ].map(({ label, rgb }) => {
    const [r, g, b] = [rgb[0] ?? 0, rgb[1] ?? 0, rgb[2] ?? 0];
    const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    return {
      label,
      x: ((b - luma) / 1.8556) * 100,
      y: -((r - luma) / 1.5748) * 100
    };
  });

  const skin = {
    x: Math.cos((SKIN_ANGLE * Math.PI) / 180) * 46,
    y: -Math.sin((SKIN_ANGLE * Math.PI) / 180) * 46
  };
</script>

<div
  class="relative mx-auto aspect-square max-w-full overflow-hidden bg-neutral-950"
  style:width="{size}px"
  role="img"
  aria-label="Vectorscope"
>
  <div class="h-full w-full origin-center" style="transform: scale({zoom})">
    <ScopeCanvas {raster} />
  </div>
  <svg
    viewBox="-50 -50 100 100"
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 h-full w-full"
  >
    <g transform="scale({zoom})" stroke-width={0.5 / zoom}>
      <circle cx="0" cy="0" r="46" fill="none" stroke="rgba(255,255,255,0.14)" />
      <line x1="-4" x2="4" y1="0" y2="0" stroke="rgba(255,255,255,0.3)" />
      <line x1="0" x2="0" y1="-4" y2="4" stroke="rgba(255,255,255,0.3)" />
      <line
        x1="0"
        x2={skin.x}
        y1="0"
        y2={skin.y}
        stroke="rgba(255,255,255,0.35)"
        stroke-dasharray="2 2"
      />
      {#each targets as target (target.label)}
        <rect
          x={target.x - BOX}
          y={target.y - BOX}
          width={BOX * 2}
          height={BOX * 2}
          fill="none"
          stroke="rgba(255,255,255,0.45)"
        />
        <text
          x={target.x + BOX + 1}
          y={target.y - BOX - 1}
          font-size={5 / zoom}
          fill="rgba(255,255,255,0.45)"
        >
          {target.label}
        </text>
      {/each}
    </g>
  </svg>
</div>
