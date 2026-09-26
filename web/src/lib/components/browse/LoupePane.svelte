<script lang="ts">
  import { observeSize } from '$lib/actions/observeSize';
  import { assetThumbUrl } from '$lib/api/assets';
  import { persistedPreviewUrl } from '$lib/api/preview';
  import { ui } from '$lib/stores/ui.svelte';
  import { CENTERED, type PaneView } from '$lib/stores/compare.svelte';
  import { fitScale, nativeScale } from '$lib/utils/view-geometry';
  import { clampZoom } from '$lib/utils/zoomLevel';

  const DRAG_THRESHOLD = 5;
  const WHEEL_STEP = 1.1;
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
    badge?: string;
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
  let loadedId = $state<string | null>(null);
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
  const loading = $derived(loadedId !== assetId);
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

  function onLoad(id: string): void {
    loadedId = id;
    measure();
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

  export function panBy(fx: number, fy: number): void {
    if (!zoomed || !zoomBox) return;
    onView(
      clampCenter({
        zoom: view.zoom,
        cx: boundedView.cx + (fx * box.w) / zoomBox.w,
        cy: boundedView.cy + (fy * box.h) / zoomBox.h
      })
    );
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

  function zoomAtPointer(e: WheelEvent): void {
    const current = zoomBox;
    const next = clampZoom(
      (view.zoom ?? fitZoom) * (e.deltaY > 0 ? 1 / WHEEL_STEP : WHEEL_STEP),
      fitZoom
    );
    const nextBox = imageBoxAt(next);
    if (next <= fitZoom || !current || !nextBox || !container) {
      onView(CENTERED, e.altKey);
      return;
    }
    const rect = container.getBoundingClientRect();
    const dx = e.clientX - (rect.left + rect.width / 2);
    const dy = e.clientY - (rect.top + rect.height / 2);
    const u = boundedView.cx + dx / current.w;
    const v = boundedView.cy + dy / current.h;
    onView(clampCenter({ zoom: next, cx: u - dx / nextBox.w, cy: v - dy / nextBox.h }), e.altKey);
  }

  function onWheel(e: WheelEvent): void {
    if (e.ctrlKey || e.metaKey) {
      e.preventDefault();
      zoomAtPointer(e);
      return;
    }
    if (!zoomed || !zoomBox) return;
    e.preventDefault();
    onView(
      clampCenter({
        zoom: view.zoom,
        cx: boundedView.cx + e.deltaX / zoomBox.w,
        cy: boundedView.cy + e.deltaY / zoomBox.h
      }),
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
  aria-busy={loading}
  class="relative flex min-h-0 min-w-0 flex-1 items-center justify-center overflow-hidden rounded-sm bg-image-canvas outline-none transition-shadow {zoomed
    ? dragging
      ? 'cursor-grabbing'
      : 'cursor-grab'
    : 'cursor-zoom-in'} {showFocus
    ? focused
      ? 'border-2 border-primary'
      : 'border-2 border-white/20 hover:border-white/35 focus-visible:border-primary'
    : 'focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary'}"
  onpointerdown={onPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  onpointercancel={onPointerUp}
  onwheel={onWheel}
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
    {#if loading}
      <img
        src={assetThumbUrl(assetId)}
        alt=""
        draggable="false"
        data-testid="loupe-underlay"
        class="pointer-events-none absolute inset-0 h-full w-full object-contain blur-sm select-none"
      />
    {/if}
    {#key assetId}
      <img
        bind:this={image}
        {src}
        {alt}
        draggable="false"
        class="relative max-w-full max-h-full object-contain select-none"
        style={transform}
        onload={() => onLoad(assetId)}
      />
    {/key}
  {/if}
</div>
