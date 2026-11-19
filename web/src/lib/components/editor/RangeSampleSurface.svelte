<script lang="ts">
  import { PhotoPick } from '#lib/editor/photoPick.svelte.ts';
  import { areaSample, pointSample } from '#lib/editor/rangeSample.ts';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import {
    surfacePixels,
    surfaceSize,
    type PreviewSurface,
    type UvRect
  } from '#lib/utils/previewSurface.ts';
  import TouchMagnifier from './TouchMagnifier.svelte';

  const POINT_PX = 5;

  let { img }: { img: PreviewSurface | null } = $props();

  const rect = imageRect(() => img);
  const picker = $derived(editor.rangePicker);
  const kind = $derived.by(() => {
    if (!picker) return null;
    const layer = editor.edits.masks.find((item) => item.id === picker.layerId);
    return layer?.components.find((item) => item.id === picker.componentId)?.kind.kind ?? null;
  });
  const active = $derived(!!picker?.ready && rect.w > 0 && rect.h > 0 && !!img);
  const noun = $derived(kind === 'luma_range' ? 'luminance' : 'color');

  function pointBox(box: UvRect): UvRect {
    if (!img) return box;
    const { w, h } = surfaceSize(img);
    const du = POINT_PX / Math.max(w, 1);
    const dv = POINT_PX / Math.max(h, 1);
    return { u: box.u - du / 2, v: box.v - dv / 2, w: du, h: dv };
  }

  function isArea(box: UvRect): boolean {
    return box.w > 0 || box.h > 0;
  }

  const pick = new PhotoPick({
    track: (box) => {
      if (!img || isArea(box)) return;
      const pixels = surfacePixels(img, pointBox(box));
      editor.previewRangeSample(pixels ? pointSample(pixels).rgb : null);
    },
    pick: (box, e) => {
      if (!img) return;
      const area = isArea(box);
      const pixels = surfacePixels(img, area ? box : pointBox(box));
      if (!pixels) return;
      void editor.commitRangeSample(area ? areaSample(pixels) : pointSample(pixels), {
        add: held('extend', e),
        keep: e.pointerType !== 'touch'
      });
    }
  });

  const pointer = $derived(pick.pointer);
  const box = $derived(pick.box);

  $effect(() => {
    if (!active) return;
    return () => pick.reset();
  });

  function onPointerLeave(): void {
    if (pick.leave()) editor.previewRangeSample(null);
  }
</script>

{#if active}
  <button
    type="button"
    data-testid="range-sample-surface"
    class="absolute z-30 cursor-crosshair bg-transparent"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none;"
    aria-label="Sample mask {noun}"
    {...pick.handlers}
    onpointerleave={onPointerLeave}
  >
    {#if pick.dragging && box}
      <span
        data-testid="range-sample-box"
        class="pointer-events-none absolute block border-2 border-dashed border-image-accent bg-image-accent/15"
        style="left: {box.u * 100}%; top: {box.v * 100}%; width: {box.w * 100}%; height: {box.h *
          100}%;"
      ></span>
    {/if}
    {#if pick.touching && pointer && img}
      <TouchMagnifier source={img} u={pointer.u} v={pointer.v} width={rect.w} zoom={2} />
    {/if}
  </button>
{/if}
