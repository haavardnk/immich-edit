<script lang="ts">
  import { cropGuides, guidePoints, TURNING_GRIDS } from '#lib/editor/cropGuides.ts';
  import type { CropGrid } from '#lib/stores/ui.svelte.ts';

  let { grid, turn, aspect }: { grid: CropGrid; turn: number; aspect: number } = $props();

  const paths = $derived(cropGuides(grid, turn, aspect));
</script>

<svg
  class="pointer-events-none absolute inset-0 size-full"
  viewBox="0 0 1 1"
  preserveAspectRatio="none"
  data-testid="crop-guide"
  data-mode={grid}
  data-turn={TURNING_GRIDS.has(grid) ? turn : undefined}
  aria-hidden="true"
>
  {#each paths as path, i (i)}
    <polyline
      points={guidePoints(path)}
      fill="none"
      stroke="rgb(255 255 255 / 0.3)"
      stroke-width="1"
      vector-effect="non-scaling-stroke"
    />
  {/each}
</svg>
