<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { maskOverlay, type MaskOverlayMode } from '#lib/stores/maskOverlay.svelte.ts';
  import { maskOverlayRgb } from '#lib/utils/brush.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';

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

  const mode = $derived(maskOverlay.mode);

  $effect(() => {
    if (!show || !canvasEl || !frame) return;
    const _color = maskOverlay.color;
    void draw(canvasEl, frame.blob, mode, maskOverlayRgb(canvasEl));
  });

  async function draw(
    canvas: HTMLCanvasElement,
    blob: Blob,
    look: MaskOverlayMode,
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
    const white = look === 'whiteOnBlack';
    for (let i = 0; i < px.length; i += 4) {
      const weight = px[i] ?? 0;
      px[i] = white ? weight : r;
      px[i + 1] = white ? weight : g;
      px[i + 2] = white ? weight : b;
      px[i + 3] = white ? 255 : weight;
    }
    ctx.putImageData(data, 0, 0);
  }
</script>

{#if show}
  <div
    class="pointer-events-none absolute"
    class:backdrop-grayscale={mode === 'colorOnBw'}
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px;"
    data-testid="mask-weight-backdrop"
    data-mode={mode}
  >
    <canvas
      bind:this={canvasEl}
      class="size-full {mode === 'whiteOnBlack' ? '' : 'opacity-55'}"
      data-testid="mask-weight"
      data-layer={layer?.id}
      aria-hidden="true"
    ></canvas>
  </div>
{/if}
