<script lang="ts">
  import type { Point } from '#lib/utils/perspective.ts';
  import type { Snippet } from 'svelte';

  let { at, width, height, photo }: { at: Point; width: number; height: number; photo: Snippet } =
    $props();

  const SIZE = 96;
  const ZOOM = 2;
  const LIFT = 72;
  const radius = SIZE / 2;
  const above = $derived(at[1] - LIFT - radius >= 0);
  const top = $derived(above ? at[1] - LIFT - SIZE : at[1] + LIFT);
</script>

<div
  class="pointer-events-none absolute z-30 overflow-hidden rounded-full bg-black shadow-lg ring-2 ring-white"
  style="width: {SIZE}px; height: {SIZE}px; left: {at[0] - radius}px; top: {top}px;"
  data-testid="guide-loupe"
  aria-hidden="true"
>
  <div
    class="absolute origin-top-left"
    style="width: {width}px; height: {height}px; transform: translate({radius -
      at[0] * ZOOM}px, {radius - at[1] * ZOOM}px) scale({ZOOM});"
  >
    {@render photo()}
  </div>
  <span
    class="absolute top-1/2 left-1/2 size-2 -translate-1/2 rounded-full bg-white ring-1 ring-black/60"
  ></span>
</div>
