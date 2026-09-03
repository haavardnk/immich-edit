<script lang="ts">
  import { observeSize } from '$lib/actions/observeSize';
  import { persistedPreviewUrl } from '$lib/api/preview';
  import { ui } from '$lib/stores/ui.svelte';
  import { CENTERED, type PaneView } from '$lib/stores/compare.svelte';
  import { fitScale, nativeScale } from '$lib/utils/view-geometry';

  const DRAG_THRESHOLD = 5;
  const MAX_SIZE = 2560;
  const SIZES = [768, 1024, 1536, 2048, MAX_SIZE];

  let {
    assetId,
    alt,
    view,
    focused = false,
    showFocus = false,
    badge,
    onView,
    onFocus,
    onSize,
    sourceLong,
    onFitZoom,
    onImage
  }: {
    assetId: string;
    alt: string;
    view: PaneView;
    focused?: boolean;
    showFocus?: boolean;
    badge?: number;
    onView: (view: PaneView, solo?: boolean) => void;
    onFocus?: () => void;
    onSize?: (maxEdge: number) => void;
    sourceLong?: number | null;
    onFitZoom?: (zoom: number) => void;
    onImage?: (element: HTMLImageElement) => void;
  } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let image = $state<HTMLImageElement | null>(null);
  let box = $state({ w: 0, h: 0 });
  let natural = $state({ w: 0, h: 0 });
  let dragging = $state(false);
  let lastX = 0;
  let lastY = 0;
  let totalDrag = 0;
  let wasFocused = false;

  const dpr = typeof window === 'undefined' ? 1 : Math.min(2, window.devicePixelRatio || 1);
  const fit = $derived(fitScale(box.w, box.h, natural.w, natural.h));
  const unit = $derived(nativeScale(Math.max(natural.w, natural.h), sourceLong ?? 0, dpr) || fit);
  const fitZoom = $derived(unit > 0 ? (100 * fit) / unit : 100);
  const zoomed = $derived(view.zoom !== null && view.zoom > fitZoom);
  const fitRatio = $derived(fit > 0 ? scaleFor(view.zoom) / fit : 1);
  const maxEdge = $derived(quantize(box.w * dpr * Math.max(1, fitRatio)));
  const src = $derived(persistedPreviewUrl(assetId, maxEdge, ui.clipWarn));
  const zoomBox = $derived.by(() => {
    return imageBoxAt(view.zoom);
  });
  const boundedView = $derived(clampCenter(view));
  const transform = $derived.by(() => {
    if (fitRatio === 1) return '';
    if (!zoomBox) return `transform: scale(${fitRatio}); transform-origin: center;`;
    const offsetX = (0.5 - boundedView.cx) * zoomBox.w;
    const offsetY = (0.5 - boundedView.cy) * zoomBox.h;
    return `transform: scale(${fitRatio}) translate(${offsetX / fitRatio}px, ${offsetY / fitRatio}px); transform-origin: center;`;
  });

  function scaleFor(zoom: number | null): number {
    return zoom === null ? fit : (zoom / 100) * unit;
  }

  function imageBoxAt(zoom: number | null): { w: number; h: number } | null {
    if (!box.w || !box.h || !natural.w || !natural.h) return null;
    const scale = scaleFor(zoom);
    if (scale <= 0) return null;
    return { w: natural.w * scale, h: natural.h * scale };
  }

  function quantize(value: number): number {
    return SIZES.find((size) => size >= value) ?? MAX_SIZE;
  }

  function clampCenter(next: PaneView): PaneView {
    const nextBox = imageBoxAt(next.zoom);
    if (!nextBox) return next;
    const limitX = Math.max(0, (nextBox.w - box.w) / 2) / nextBox.w;
    const limitY = Math.max(0, (nextBox.h - box.h) / 2) / nextBox.h;
    return {
      zoom: next.zoom,
      cx: Math.min(0.5 + limitX, Math.max(0.5 - limitX, next.cx)),
      cy: Math.min(0.5 + limitY, Math.max(0.5 - limitY, next.cy))
    };
  }

  function measure(): void {
    if (container) box = { w: container.clientWidth, h: container.clientHeight };
    if (image) {
      natural = { w: image.naturalWidth, h: image.naturalHeight };
      onImage?.(image);
    }
  }

  function zoomInAt(clientX: number, clientY: number, solo: boolean): void {
    const zoom = ui.zoomLevel;
    if (!zoomBox || !container) {
      onView({ zoom, cx: 0.5, cy: 0.5 }, solo);
      return;
    }
    const rect = container.getBoundingClientRect();
    const cx = 0.5 + (clientX - (rect.left + rect.width / 2)) / zoomBox.w;
    const cy = 0.5 + (clientY - (rect.top + rect.height / 2)) / zoomBox.h;
    onView(clampCenter({ zoom, cx, cy }), solo);
  }

  function onPointerDown(e: PointerEvent): void {
    e.preventDefault();
    wasFocused = focused || !showFocus;
    onFocus?.();
    lastX = e.clientX;
    lastY = e.clientY;
    totalDrag = 0;
    if (zoomed) {
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
    onView(
      clampCenter({ zoom: view.zoom, cx: view.cx - dx / zoomBox.w, cy: view.cy - dy / zoomBox.h }),
      e.altKey
    );
  }

  function onPointerUp(e: PointerEvent): void {
    const wasDragging = dragging;
    dragging = false;
    if (wasDragging && totalDrag > DRAG_THRESHOLD) return;
    if (!wasFocused) return;
    if (zoomed) onView(CENTERED, e.altKey);
    else zoomInAt(e.clientX, e.clientY, e.altKey);
  }

  $effect(() => {
    onSize?.(maxEdge);
  });

  $effect(() => {
    onFitZoom?.(fitZoom);
  });
</script>

<div
  bind:this={container}
  use:observeSize={measure}
  role="button"
  tabindex="0"
  aria-label={zoomed ? 'Zoom out' : 'Zoom in'}
  class="flex min-h-0 min-w-0 flex-1 items-center justify-center overflow-hidden rounded-sm bg-image-canvas outline-none transition-shadow focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary {zoomed
    ? dragging
      ? 'cursor-grabbing'
      : 'cursor-grab'
    : 'cursor-zoom-in'} {showFocus
    ? focused
      ? 'relative border-2 border-primary'
      : 'relative border-2 border-white/20 hover:border-white/35'
    : ''}"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
>
  {#if badge !== undefined}
    <span
      class="pointer-events-none absolute top-2 left-2 z-10 min-w-6 rounded-md border px-1.5 text-center text-[11px] leading-5 font-semibold shadow-md backdrop-blur-sm {focused
        ? 'border-primary bg-primary text-neutral-950'
        : 'border-white/20 bg-neutral-950/85 text-white/90'}"
    >
      {badge}
    </span>
  {/if}
  {#if box.w > 0}
    <img
      bind:this={image}
      {src}
      {alt}
      draggable="false"
      class="max-w-full max-h-full object-contain select-none"
      style={transform}
      onload={measure}
    />
  {/if}
</div>
