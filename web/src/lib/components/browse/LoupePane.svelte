<script lang="ts">
  import { persistedPreviewUrl } from '$lib/api/preview';
  import type { PaneView } from '$lib/stores/compare.svelte';

  const ZOOM = 2.5;
  const DRAG_THRESHOLD = 5;
  const SIZES = [768, 1024, 1536, 2048, 2560];

  let {
    assetId,
    alt,
    view,
    focused = false,
    showFocus = false,
    onView,
    onFocus,
    onSize
  }: {
    assetId: string;
    alt: string;
    view: PaneView;
    focused?: boolean;
    showFocus?: boolean;
    onView: (view: PaneView) => void;
    onFocus?: () => void;
    onSize?: (maxEdge: number) => void;
  } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let image = $state<HTMLImageElement | null>(null);
  let box = $state({ w: 0, h: 0 });
  let natural = $state({ w: 0, h: 0 });
  let dragging = $state(false);
  let lastX = 0;
  let lastY = 0;
  let totalDrag = 0;

  const scale = typeof window === 'undefined' ? 1 : Math.min(2, window.devicePixelRatio || 1);
  const maxEdge = $derived(quantize(box.w * scale * (view.zoomed ? ZOOM : 1)));
  const src = $derived(persistedPreviewUrl(assetId, maxEdge));
  const zoomBox = $derived.by(() => {
    if (!box.w || !box.h || !natural.w || !natural.h) return null;
    const fit = Math.min(box.w / natural.w, box.h / natural.h);
    return { w: natural.w * fit * ZOOM, h: natural.h * fit * ZOOM };
  });
  const transform = $derived.by(() => {
    if (!view.zoomed) return '';
    if (!zoomBox) return `transform: scale(${ZOOM}); transform-origin: center;`;
    const offsetX = (0.5 - view.cx) * zoomBox.w;
    const offsetY = (0.5 - view.cy) * zoomBox.h;
    return `transform: scale(${ZOOM}) translate(${offsetX / ZOOM}px, ${offsetY / ZOOM}px); transform-origin: center;`;
  });

  function quantize(value: number): number {
    return SIZES.find((size) => size >= value) ?? SIZES[SIZES.length - 1];
  }

  function clampCenter(next: PaneView): PaneView {
    if (!zoomBox) return next;
    const limitX = Math.max(0, (zoomBox.w - box.w) / 2) / zoomBox.w;
    const limitY = Math.max(0, (zoomBox.h - box.h) / 2) / zoomBox.h;
    return {
      zoomed: next.zoomed,
      cx: Math.min(0.5 + limitX, Math.max(0.5 - limitX, next.cx)),
      cy: Math.min(0.5 + limitY, Math.max(0.5 - limitY, next.cy))
    };
  }

  function measure(): void {
    if (container) box = { w: container.clientWidth, h: container.clientHeight };
    if (image) natural = { w: image.naturalWidth, h: image.naturalHeight };
  }

  function zoomInAt(clientX: number, clientY: number): void {
    if (!zoomBox || !container) {
      onView({ zoomed: true, cx: 0.5, cy: 0.5 });
      return;
    }
    const rect = container.getBoundingClientRect();
    const cx = 0.5 + ((clientX - (rect.left + rect.width / 2)) * ZOOM) / zoomBox.w;
    const cy = 0.5 + ((clientY - (rect.top + rect.height / 2)) * ZOOM) / zoomBox.h;
    onView(clampCenter({ zoomed: true, cx, cy }));
  }

  function onPointerDown(e: PointerEvent): void {
    e.preventDefault();
    onFocus?.();
    lastX = e.clientX;
    lastY = e.clientY;
    totalDrag = 0;
    if (view.zoomed) {
      dragging = true;
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    }
  }

  function onPointerMove(e: PointerEvent): void {
    if (!dragging || !zoomBox) return;
    const dx = e.clientX - lastX;
    const dy = e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
    totalDrag += Math.abs(dx) + Math.abs(dy);
    onView(clampCenter({ zoomed: true, cx: view.cx - dx / zoomBox.w, cy: view.cy - dy / zoomBox.h }));
  }

  function onPointerUp(e: PointerEvent): void {
    const wasDragging = dragging;
    dragging = false;
    if (wasDragging && totalDrag > DRAG_THRESHOLD) return;
    if (view.zoomed) onView({ zoomed: false, cx: 0.5, cy: 0.5 });
    else zoomInAt(e.clientX, e.clientY);
  }

  $effect(() => {
    if (!container) return;
    measure();
    const observer = new ResizeObserver(() => measure());
    observer.observe(container);
    return () => observer.disconnect();
  });

  $effect(() => {
    onSize?.(maxEdge);
  });
</script>

<div
  bind:this={container}
  role="button"
  tabindex="0"
  aria-label={view.zoomed ? 'Zoom out' : 'Zoom in'}
  class="flex-1 min-w-0 min-h-0 flex items-center justify-center overflow-hidden {view.zoomed
    ? dragging
      ? 'cursor-grabbing'
      : 'cursor-grab'
    : 'cursor-zoom-in'} {showFocus
    ? focused
      ? 'ring-2 ring-immich-dark-primary rounded-lg'
      : 'ring-1 ring-white/10 rounded-lg'
    : ''}"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
>
  {#if box.w > 0}
    <img
      bind:this={image}
      src={src}
      {alt}
      draggable="false"
      class="max-w-full max-h-full object-contain select-none"
      style={transform}
      onload={measure}
    />
  {/if}
</div>
