<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { parseHexColor } from '$lib/utils/brush';
  import { imageRect } from '$lib/utils/imageRect.svelte';
  import type { PreviewSurface } from '$lib/utils/previewSurface';

  let {
    img
  }: {
    img: PreviewSurface | null;
  } = $props();

  const rect = imageRect(() => img);
  let canvasEl = $state<HTMLCanvasElement | null>(null);

  const layer = $derived(
    editor.maskOverlayLayerId
      ? (editor.edits.masks.find((l) => l.id === editor.maskOverlayLayerId) ?? null)
      : null
  );
  const frame = $derived(
    layer && editor.maskWeight?.layerId === layer.id ? editor.maskWeight : null
  );
  const show = $derived(!!frame && editor.maskStroke === 'idle' && rect.w > 0 && rect.h > 0);

  $effect(() => {
    editor.syncMaskWeight();
  });

  $effect(() => {
    if (!show || !canvasEl || !frame || !layer) return;
    void draw(canvasEl, frame.blob, parseHexColor(layer.color));
  });

  async function draw(
    canvas: HTMLCanvasElement,
    blob: Blob,
    [r, g, b]: [number, number, number]
  ): Promise<void> {
    let bitmap: ImageBitmap;
    try {
      bitmap = await createImageBitmap(blob, { colorSpaceConversion: 'none' });
    } catch {
      return;
    }
    if (frame?.blob !== blob) {
      bitmap.close();
      return;
    }
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) {
      bitmap.close();
      return;
    }
    ctx.drawImage(bitmap, 0, 0);
    bitmap.close();
    const data = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const px = data.data;
    for (let i = 0; i < px.length; i += 4) {
      const weight = px[i] ?? 0;
      px[i] = r;
      px[i + 1] = g;
      px[i + 2] = b;
      px[i + 3] = weight;
    }
    ctx.putImageData(data, 0, 0);
  }
</script>

{#if show}
  <canvas
    bind:this={canvasEl}
    class="pointer-events-none absolute opacity-55"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px;"
    data-testid="mask-weight"
    data-layer={layer?.id}
    aria-hidden="true"
  ></canvas>
{/if}
