<script lang="ts">
  import type { Rect } from '#lib/utils/viewGeometry.ts';

  let {
    url,
    rect,
    alt,
    testid,
    class: className = '',
    style = ''
  }: {
    url: string;
    rect: Rect;
    alt: string;
    testid: string;
    class?: string;
    style?: string;
  } = $props();
</script>

<img
  src={url}
  {alt}
  data-testid={testid}
  data-photo-layer
  draggable="false"
  class="pointer-events-none absolute max-h-none max-w-none select-none {className}"
  style="left: {rect.left}px; top: {rect.top}px; width: {rect.width}px; height: {rect.height}px; image-orientation: none; {style}"
/>
