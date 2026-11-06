<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import { paneNatural, paneSources, type LoadedSize } from '#lib/browse/previewSource.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
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
  import PreviewStatusChip from '#lib/components/browse/PreviewStatusChip.svelte';

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
  let preview = $state<HTMLImageElement | null>(null);
  let box = $state({ w: 0, h: 0 });
  let rendered = $state<LoadedSize | null>(null);
  let previewed = $state<LoadedSize | null>(null);
  let failedSrc = $state<string | null>(null);
  let dragging = $state(false);
  let wasFocused = false;

  const dpr = $derived(Math.min(2, devicePixelRatio.current || 1));
  const natural = $derived(paneNatural(assetId, rendered, previewed));
  const metrics = $derived(viewportMetrics(box, natural, sourceLong ?? 0, dpr));
  const zoomed = $derived(isZoomed(metrics, view));
  const shown = $derived(clampView(metrics, view));
  const maxEdge = $derived(quantize(box.w * dpr * Math.max(1, zoomRatio(metrics, view.zoom))));
  const sources = $derived(paneSources(assetId, rendered?.id ?? null, maxEdge, ui.clipWarn));
  const loading = $derived(rendered?.id !== assetId);
  const rendering = $derived(loading && failedSrc !== sources.render);
  const transform = $derived(viewportTransform(metrics, shown));

  function quantize(value: number): number {
    return SIZES.find((size) => size >= value) ?? MAX_SIZE;
  }

  function measure(): void {
    if (container) box = { w: container.clientWidth, h: container.clientHeight };
  }

  function sizeOf(id: string, element: HTMLImageElement): LoadedSize {
    return { id, w: element.naturalWidth, h: element.naturalHeight };
  }

  function onRender(id: string): void {
    if (!image) return;
    rendered = sizeOf(id, image);
    onImage?.(image);
  }

  function onPreview(id: string): void {
    if (preview) previewed = sizeOf(id, preview);
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
    {#key assetId}
      {#if sources.underlay}
        <img
          bind:this={preview}
          src={sources.underlay}
          alt=""
          draggable="false"
          data-testid="loupe-immich-preview"
          class="pointer-events-none absolute inset-0 h-full w-full object-contain select-none"
          style={transform}
          onload={() => onPreview(assetId)}
        />
      {/if}
      <img
        bind:this={image}
        src={sources.render}
        {alt}
        draggable="false"
        class="relative max-w-full max-h-full object-contain select-none"
        style={transform}
        onload={() => onRender(assetId)}
        onerror={() => (failedSrc = sources.render)}
      />
    {/key}
  {/if}
  <PreviewStatusChip pending={rendering} edited={editedThumbs.has(assetId)} episode={assetId} />
</div>
