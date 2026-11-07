<script lang="ts">
  import { observeSize } from '#lib/actions/observeSize.ts';
  import {
    paneNatural,
    paneSources,
    type LoadedRender,
    type LoadedSize
  } from '#lib/browse/previewSource.ts';
  import { editedThumbs } from '#lib/stores/editedThumbs.svelte.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import {
    clampView,
    FIT_VIEW,
    isZoomed,
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
  import { loupeGestures } from '#lib/browse/loupeGestures.ts';
  import type { LoupeStage } from '#lib/browse/loupeStage.ts';
  import { coverageOf, rejectMark } from '#lib/browse/bulk/bulkMarks.ts';
  import { Icon } from '@immich/ui';

  const MAX_SIZE = 2560;
  const SIZES = [768, 1024, 1536, 2048, MAX_SIZE];
  const rejectedMark = rejectMark(coverageOf(true));

  let {
    assetId,
    thumbhash = null,
    checksum = null,
    alt,
    view,
    focused = false,
    showFocus = false,
    badge,
    caption,
    rejected = false,
    onView,
    onFocus,
    onSize,
    sourceLong,
    onMetrics,
    onImage,
    stage = null
  }: {
    assetId: string;
    thumbhash?: string | null;
    checksum?: string | null;
    alt: string;
    view: View;
    focused?: boolean;
    showFocus?: boolean;
    badge?: string;
    caption?: string;
    rejected?: boolean;
    onView: (view: View, solo?: boolean) => void;
    onFocus?: () => void;
    onSize?: (maxEdge: number) => void;
    sourceLong?: number | null;
    onMetrics?: (metrics: ViewportMetrics) => void;
    onImage?: (element: HTMLImageElement) => void;
    stage?: LoupeStage | null;
  } = $props();

  let container = $state<HTMLDivElement | null>(null);
  let image = $state<HTMLImageElement | null>(null);
  let preview = $state<HTMLImageElement | null>(null);
  let box = $state({ w: 0, h: 0 });
  let rendered = $state<LoadedRender | null>(null);
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
  const sources = $derived(
    paneSources({ id: assetId, thumbhash, checksum }, rendered, failedSrc, maxEdge, ui.clipWarn)
  );
  const loading = $derived(rendered?.id !== assetId);
  const rendering = $derived(loading && sources.render !== null);
  const transform = $derived(viewportTransform(metrics, shown));
  const dim = $derived(rejected ? 'opacity-40 grayscale' : '');

  function quantize(value: number): number {
    return SIZES.find((size) => size >= value) ?? MAX_SIZE;
  }

  function measure(): void {
    if (container) box = { w: container.clientWidth, h: container.clientHeight };
  }

  function sizeOf(id: string, element: HTMLImageElement): LoadedSize {
    return { id, w: element.naturalWidth, h: element.naturalHeight };
  }

  function onRender(id: string, src: string): void {
    if (!image) return;
    rendered = { ...sizeOf(id, image), src };
    onImage?.(image);
  }

  function onPreview(id: string): void {
    if (preview) previewed = sizeOf(id, preview);
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
    },
    pointerTypes: ['mouse', 'pen']
  });

  const touchGestures = loupeGestures({
    element: () => container,
    metrics: () => metrics,
    view: () => shown,
    apply: (next) => onView(next),
    focus: () => onFocus?.(),
    zoomLevel: () => ui.zoomLevel,
    stage: () => stage
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
  {@attach touchGestures}
>
  {#if badge !== undefined || rejected}
    <div class="pointer-events-none absolute top-2 left-2 z-10 flex items-center gap-1">
      {#if badge !== undefined}
        <span
          class="min-w-6 rounded-full border px-1.5 text-center text-[11px] leading-5 font-semibold shadow-md backdrop-blur-sm {focused
            ? 'border-primary bg-primary text-neutral-950'
            : 'border-white/20 bg-neutral-950/85 text-white/90'}"
        >
          {badge}
        </span>
      {/if}
      {#if caption}
        <span
          class="rounded-md border border-white/20 bg-neutral-950/85 px-1.5 text-[11px] leading-5 font-semibold text-white/90 shadow-md backdrop-blur-sm"
        >
          {caption}
        </span>
      {/if}
      {#if rejected}
        <span
          role="img"
          aria-label="Rejected"
          class="grid size-[22px] place-items-center rounded-full border border-white/20 bg-neutral-950/85 shadow-md {rejectedMark.tone}"
        >
          <Icon icon={rejectedMark.icon} size="16px" aria-hidden="true" />
        </span>
      {/if}
    </div>
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
          class="pointer-events-none absolute inset-0 h-full w-full object-contain select-none {dim}"
          style={transform}
          onload={() => onPreview(assetId)}
        />
      {/if}
      {#if sources.render}
        {@const src = sources.render}
        <img
          bind:this={image}
          {src}
          {alt}
          draggable="false"
          class="relative max-w-full max-h-full object-contain select-none {dim}"
          style={transform}
          onload={() => onRender(assetId, src)}
          onerror={() => (failedSrc = src)}
        />
      {/if}
    {/key}
  {/if}
  <PreviewStatusChip pending={rendering} edited={editedThumbs.has(assetId)} episode={assetId} />
</div>
