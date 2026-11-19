<script lang="ts">
  import CompareLabel from '#lib/components/editor/compare/CompareLabel.svelte';
  import ViewerImage from '#lib/components/viewer/ViewerImage.svelte';
  import type { Rect } from '#lib/utils/viewGeometry.ts';
  import type { ViewerLabels } from '#lib/viewer/protocol.ts';

  let {
    frame,
    left,
    current,
    alt,
    labels,
    edge
  }: {
    frame: Rect;
    left: string | null;
    current: string;
    alt: string;
    labels: ViewerLabels;
    edge: string;
  } = $props();
</script>

<div class="absolute inset-0 flex" data-compare="side">
  <div class="relative min-w-0 flex-1 overflow-hidden" data-compare-pane="left">
    {#if left}
      <ViewerImage url={left} rect={frame} alt={labels.left} testid="viewer-left" class={edge} />
    {/if}
    <CompareLabel text={labels.left} class="left-2.5" />
  </div>
  <div class="w-0.5 shrink-0 bg-hairline" aria-hidden="true"></div>
  <div class="relative min-w-0 flex-1 overflow-hidden" data-compare-pane="right">
    <ViewerImage url={current} rect={frame} {alt} testid="viewer-image" class={edge} />
    <CompareLabel text={labels.right} class="left-2.5" />
  </div>
</div>
