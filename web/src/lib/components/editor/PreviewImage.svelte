<script lang="ts">
  import { untrack } from 'svelte';
  import type { PreviewFrame } from '$lib/stores/editor/preview.svelte';
  import type { PreviewSurface } from '$lib/utils/preview-surface';

  let {
    url,
    frame,
    alt,
    class: className,
    style,
    testid,
    element = $bindable(null),
    onsize
  }: {
    url: string | null;
    frame: PreviewFrame | null;
    alt: string;
    class: string;
    style: string;
    testid?: string;
    element?: PreviewSurface | null;
    onsize?: (size: { w: number; h: number }) => void;
  } = $props();

  $effect(() => {
    const canvas = element;
    if (!frame || !(canvas instanceof HTMLCanvasElement)) return;
    const { bitmap, colorSpace } = frame;
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    canvas.getContext('2d', { alpha: false, colorSpace })?.drawImage(bitmap, 0, 0);
    untrack(() => onsize?.({ w: bitmap.width, h: bitmap.height }));
  });
</script>

{#if frame}
  {#key frame.colorSpace}
    <canvas bind:this={element} data-testid={testid} class={className} {style}>{alt}</canvas>
  {/key}
{:else if url}
  <img
    bind:this={element}
    src={url}
    {alt}
    data-testid={testid}
    class={className}
    style="{style} image-orientation: none;"
    draggable="false"
    onload={() => {
      if (element instanceof HTMLImageElement && element.naturalWidth > 0) {
        onsize?.({ w: element.naturalWidth, h: element.naturalHeight });
      }
    }}
  />
{/if}
