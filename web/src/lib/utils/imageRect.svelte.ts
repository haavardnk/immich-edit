import { ui } from '$lib/stores/ui.svelte';

export type ImageRect = { x: number; y: number; w: number; h: number };

export function imageRect(img: () => HTMLImageElement | null): ImageRect {
  const rect: ImageRect = $state({ x: 0, y: 0, w: 0, h: 0 });

  function recompute(): void {
    const el = img();
    const parent = el?.parentElement;
    if (!el || !parent) return;
    const p = parent.getBoundingClientRect();
    const r = el.getBoundingClientRect();
    rect.x = r.left - p.left;
    rect.y = r.top - p.top;
    rect.w = r.width;
    rect.h = r.height;
  }

  $effect(() => {
    const el = img();
    if (!el) return;
    recompute();
    const ro = new ResizeObserver(recompute);
    ro.observe(el);
    if (el.parentElement) ro.observe(el.parentElement);
    el.addEventListener('load', recompute);
    window.addEventListener('resize', recompute);
    return () => {
      ro.disconnect();
      el.removeEventListener('load', recompute);
      window.removeEventListener('resize', recompute);
    };
  });

  $effect(() => {
    void ui.zoom;
    void ui.panX;
    void ui.panY;
    if (!img()) return;
    const id = requestAnimationFrame(recompute);
    return () => cancelAnimationFrame(id);
  });

  return rect;
}
