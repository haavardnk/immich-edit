<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import { assetThumbUrl } from '#lib/api/assets.ts';
  import { persistedPreviewUrl } from '#lib/api/preview.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import {
    clampView,
    FIT_VIEW,
    isZoomed,
    panByStep,
    viewportMetrics,
    viewportTransform,
    zoomRatio,
    type View,
    type ViewportMetrics
  } from '#lib/utils/imageViewport.ts';
  import { untrack } from 'svelte';
  import { devicePixelRatio } from 'svelte/reactivity/window';
  import { held } from '#lib/input/modifiers.svelte.ts';
  import { viewportGestures } from '#lib/gestures/viewportGestures.ts';

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
    onMetrics,
    onImage
  }: {
    assetId: string;
    alt: string;
    view: View;
    focused?: boolean;
    showFocus?: boolean;
    badge?: string;
    onView: (view: View, solo?: boolean) => void;
    onFocus?: () => void;
    onSize?: (maxEdge: number) => void;
    sourceLong?: number | null;
    onMetrics?: (metrics: ViewportMetrics) => void;
    onImage?: (element: HTMLImageElement) => void;
  } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let image = $state<HTMLImageElement | null>(null);
  let box = $state({ w: 0, h: 0 });
  let natural = $state({ w: 0, h: 0 });
  let dragging = $state(false);
  let loadedId = $state<string | null>(null);
  let wasFocused = false;

  const dpr = $derived(Math.min(2, devicePixelRatio.current || 1));
  const metrics = $derived(viewportMetrics(box, natural, sourceLong ?? 0, dpr));
  const zoomed = $derived(isZoomed(metrics, view));
  const shown = $derived(clampView(metrics, view));
  const maxEdge = $derived(quantize(box.w * dpr * Math.max(1, zoomRatio(metrics, view.zoom))));
  const src = $derived(persistedPreviewUrl(assetId, maxEdge, ui.clipWarn));
  const loading = $derived(loadedId !== assetId);
  const transform = $derived(viewportTransform(metrics, shown));

  function quantize(value: number): number {
    return SIZES.find((size) => size >= value) ?? MAX_SIZE;
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

  export function panBy(fx: number, fy: number): void {
    if (!zoomed) return;
    onView(panByStep(metrics, shown, [fx, fy]));
  }

  const gestures = viewportGestures({
    element: () => container,
    metrics: () => metrics,
    view: () => shown,
    apply: (next, e) => onView(next, e !== undefined && held('solo', e)),
    press(e) {
      e.preventDefault();
      wasFocused = focused || !showFocus;
      onFocus?.();
    },
    dragging: (active) => (dragging = active),
    tap(e) {
      if (!wasFocused) return;
      if (zoomed) onView(FIT_VIEW, held('solo', e));
      else gestures.zoomAt(e, ui.zoomLevel, e);
    }
  });

  $effect(() => {
    onSize?.(maxEdge);
  });

  $effect(() => {
    const current = metrics;
    untrack(() => onMetrics?.(current));
  });
</script>

<div
  bind:this={container}
  use:observeSize={measure}
  role="button"
  tabindex="0"
  aria-label={zoomed ? 'Zoom out' : 'Zoom in'}
  aria-busy={loading}
  class="no-callout relative flex min-h-0 min-w-0 flex-1 items-center justify-center overflow-hidden rounded-sm bg-image-canvas outline-none transition-shadow {zoomed
    ? dragging
      ? 'cursor-grabbing'
      : 'cursor-grab'
    : 'cursor-zoom-in'} {showFocus
    ? focused
      ? 'border-2 border-primary'
      : 'border-2 border-white/20 hover:border-white/35 focus-visible:border-primary'
    : 'focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-primary'}"
  {...gestures.handlers}
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
