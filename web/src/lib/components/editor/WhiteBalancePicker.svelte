<script lang="ts">
  import { editor } from '$lib/stores/editor.svelte';
  import { clamp01 } from '$lib/utils/geom';
  import { imageRect } from '$lib/utils/imageRect.svelte';

  let {
    img
  }: {
    img: HTMLImageElement | null;
  } = $props();

  const rect = imageRect(() => img);
  const active = $derived(editor.wbPicking && rect.w > 0 && rect.h > 0 && !!img);

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== 'Escape' || !editor.wbPicking) return;
    e.preventDefault();
    editor.cancelWbPicker();
  }

  $effect(() => {
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  function pick(e: PointerEvent): void {
    e.preventDefault();
    e.stopPropagation();
    if (!img) return;
    const b = img.getBoundingClientRect();
    const u = clamp01((e.clientX - b.left) / Math.max(b.width, 1));
    const v = clamp01((e.clientY - b.top) / Math.max(b.height, 1));
    void editor.pickWhiteBalance(u, v);
  }
</script>

{#if active}
  <button
    type="button"
    data-testid="wb-picker-surface"
    class="absolute z-30 cursor-crosshair bg-transparent"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px;"
    aria-label="Sample neutral color"
    onpointerdown={pick}
  ></button>
{/if}
