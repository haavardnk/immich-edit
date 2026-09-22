<script lang="ts">
  import ScopeCanvas from './ScopeCanvas.svelte';
  import { rasterize } from './rasterize';
  import type { ScopeGrid } from '$lib/types/preview';

  let {
    grid,
    gain,
    parade,
    label,
    height
  }: { grid: ScopeGrid; gain: number; parade: boolean; label: string; height: number } = $props();

  const raster = $derived(rasterize(grid, gain, parade));
  const levels = [0, 25, 50, 75, 100];
</script>

<div class="relative w-full bg-neutral-950" style:height="{height}px" role="img" aria-label={label}>
  <ScopeCanvas {raster} />
  <svg
    viewBox="0 0 100 100"
    preserveAspectRatio="none"
    aria-hidden="true"
    class="pointer-events-none absolute inset-0 h-full w-full"
  >
    {#each levels as level (level)}
      <line
        x1="0"
        x2="100"
        y1={100 - level}
        y2={100 - level}
        stroke="rgba(255,255,255,0.14)"
        stroke-width="0.4"
      />
    {/each}
    {#if parade}
      <line
        x1="33.33"
        x2="33.33"
        y1="0"
        y2="100"
        stroke="rgba(255,255,255,0.2)"
        stroke-width="0.4"
      />
      <line
        x1="66.66"
        x2="66.66"
        y1="0"
        y2="100"
        stroke="rgba(255,255,255,0.2)"
        stroke-width="0.4"
      />
    {/if}
  </svg>
  <div
    class="pointer-events-none absolute inset-y-0 left-0.5 flex flex-col justify-between py-px text-[8px] font-mono text-dark/45"
  >
    {#each [...levels].reverse() as level (level)}
      <span>{level}</span>
    {/each}
  </div>
</div>
