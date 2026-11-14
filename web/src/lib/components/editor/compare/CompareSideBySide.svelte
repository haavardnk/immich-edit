<script lang="ts">
  import PreviewImage from '#lib/components/editor/PreviewImage.svelte';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import CompareLabel from './CompareLabel.svelte';
  import { leftLabel } from './compareText.ts';

  let {
    surface = $bindable(null),
    frame,
    edge,
    onsize
  }: {
    surface?: PreviewSurface | null;
    frame: string;
    edge: string;
    onsize: (size: { w: number; h: number }) => void;
  } = $props();

  const label = $derived(leftLabel());
  const image = $derived(`max-h-none max-w-none select-none object-contain ${edge}`);
</script>

<div class="absolute inset-0 flex" data-compare="side">
  <div
    class="relative flex min-w-0 flex-1 items-center justify-center overflow-hidden"
    data-photo-layer
    data-compare-pane="left"
  >
    {#if editor.compareUrl}
      <img
        src={editor.compareUrl}
        alt={label}
        data-photo-layer
        draggable="false"
        class={image}
        style="{frame} image-orientation: none;"
      />
    {/if}
    <CompareLabel text={label} class="left-2.5" />
  </div>
  <div class="w-0.5 shrink-0 bg-hairline" aria-hidden="true"></div>
  <div
    class="relative flex min-w-0 flex-1 items-center justify-center overflow-hidden"
    data-photo-layer
    data-compare-pane="right"
  >
    <PreviewImage
      bind:element={surface}
      url={editor.previewUrl}
      frame={editor.previewFrame}
      alt={editor.asset?.originalFileName ?? ''}
      class={image}
      style={frame}
      testid="preview-image"
      {onsize}
    />
    <CompareLabel text="Current" class="left-2.5" />
  </div>
</div>
