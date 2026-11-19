import { ui } from '#lib/stores/ui.svelte.ts';
import type { PreviewSurface } from './previewSurface';

export type ImageRect = { x: number; y: number; w: number; h: number };

export function imageRect(img: () => PreviewSurface | null): ImageRect {
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
    let frame = 0;
    const schedule = (): void => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(recompute);
    };
    const ro = new ResizeObserver(schedule);
    ro.observe(el);
    if (el.parentElement) ro.observe(el.parentElement);
    const moved = new MutationObserver(schedule);
    moved.observe(el, { attributes: true, attributeFilter: ['style', 'class'] });
    el.addEventListener('load', schedule);
    el.addEventListener('transitionend', schedule);
    window.addEventListener('resize', schedule);
    return () => {
      cancelAnimationFrame(frame);
      ro.disconnect();
      moved.disconnect();
      el.removeEventListener('load', schedule);
      el.removeEventListener('transitionend', schedule);
      window.removeEventListener('resize', schedule);
    };
  });

  $effect(() => {
    void ui.zoom;
    void ui.cx;
    void ui.cy;
    if (!img()) return;
    const id = requestAnimationFrame(recompute);
    return () => cancelAnimationFrame(id);
  });

  return rect;
}
