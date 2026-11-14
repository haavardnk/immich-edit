<script lang="ts">
  import CompareLabel from '#lib/components/editor/compare/CompareLabel.svelte';
  import type { Rect } from '#lib/utils/viewGeometry.ts';
  import type { ViewerLabels } from '#lib/viewer/protocol.ts';

  let {
    frame,
    left,
    current,
    alt,
    split,
    labels,
    edge
  }: {
    frame: Rect;
    left: string | null;
    current: string;
    alt: string;
    split: number;
    labels: ViewerLabels;
    edge: string;
  } = $props();

  const at = $derived(`${split * 100}%`);
</script>

<div
  data-photo-layer
  data-compare="split"
  class="pointer-events-none absolute overflow-hidden {edge}"
  style="left: {frame.left}px; top: {frame.top}px; width: {frame.width}px; height: {frame.height}px; --split: {at};"
>
  {#if left}
    <img
      src={left}
      alt={labels.left}
      data-testid="viewer-left"
      draggable="false"
      class="absolute inset-0 h-full w-full object-contain select-none"
      style="image-orientation: none;"
    />
  {/if}
  <img
    src={current}
    {alt}
    data-testid="viewer-image"
    draggable="false"
    class="absolute inset-0 h-full w-full object-contain select-none"
    style="clip-path: inset(0 0 0 {at}); image-orientation: none;"
  />
  <div
    aria-hidden="true"
    class="absolute top-0 bottom-0 w-0.5 -translate-x-1/2 bg-white/90 shadow-split"
    style="left: {at};"
  ></div>
  <div
    aria-hidden="true"
    class="absolute top-1/2 h-5 w-5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-white shadow-lg"
    style="left: {at};"
  ></div>
  <CompareLabel text={labels.left} class="left-2.5" />
  <CompareLabel text={labels.right} class="left-[calc(var(--split)+0.625rem)]" />
</div>
