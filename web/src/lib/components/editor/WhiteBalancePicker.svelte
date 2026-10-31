<script lang="ts">
  import type { WhiteBalanceSample } from '#lib/api/edits.ts';
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { WhiteBalanceProbe } from '#lib/stores/editor/whiteBalanceProbe.svelte.ts';
  import type { Edits } from '#lib/types/edits.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { clamp01 } from '#lib/utils/geom.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';

  type Spot = { u: number; v: number; x: number; y: number; id: number };

  const MIN_DRAG_PX = 6;

  let {
    img
  }: {
    img: PreviewSurface | null;
  } = $props();

  const rect = imageRect(() => img);
  const active = $derived(editor.wbPicking && rect.w > 0 && rect.h > 0 && !!img);
  const probe = new WhiteBalanceProbe(() =>
    editor.assetId
      ? { assetId: editor.assetId, edits: $state.snapshot(editor.edits) as Edits }
      : null
  );
  let start = $state<Spot | null>(null);
  let pointer = $state<Spot | null>(null);

  const dragging = $derived(
    !!start && !!pointer && Math.hypot(pointer.x - start.x, pointer.y - start.y) >= MIN_DRAG_PX
  );
  const sample = $derived<WhiteBalanceSample | null>(
    start && dragging && pointer
      ? boxSample(start, pointer)
      : pointer
        ? boxSample(pointer, pointer)
        : null
  );
  const reading = $derived(probe.reading);
  const cardShift = $derived(
    pointer
      ? `${pointer.u > 0.7 ? 'calc(-100% - 16px)' : '16px'}, ${pointer.v > 0.8 ? 'calc(-100% - 16px)' : '16px'}`
      : ''
  );

  $effect(() => {
    if (!active) return;
    return () => {
      start = null;
      pointer = null;
      probe.clear();
    };
  });

  function boxSample(a: Spot, b: Spot): WhiteBalanceSample {
    const u = Math.min(a.u, b.u);
    const v = Math.min(a.v, b.v);
    return { u, v, w: Math.max(a.u, b.u) - u, h: Math.max(a.v, b.v) - v };
  }

  function spotAt(e: PointerEvent): Spot {
    const b = (e.currentTarget as Element).getBoundingClientRect();
    return {
      u: clamp01((e.clientX - b.left) / Math.max(b.width, 1)),
      v: clamp01((e.clientY - b.top) / Math.max(b.height, 1)),
      x: e.clientX,
      y: e.clientY,
      id: e.pointerId
    };
  }

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== 'Escape' || !editor.wbPicking) return;
    e.preventDefault();
    editor.cancelWbPicker();
  }

  function onPointerDown(e: PointerEvent): void {
    e.preventDefault();
    e.stopPropagation();
    if (editor.wbBusy || start || e.button !== 0) return;
    start = spotAt(e);
    pointer = start;
    (e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId);
  }

  function onPointerMove(e: PointerEvent): void {
    if (start && e.pointerId !== start.id) return;
    pointer = spotAt(e);
    if (sample && !editor.wbBusy) probe.probe(sample);
  }

  function onPointerUp(e: PointerEvent): void {
    if (!start || e.pointerId !== start.id) return;
    e.stopPropagation();
    (e.currentTarget as HTMLElement).releasePointerCapture?.(e.pointerId);
    const picked = dragging && pointer ? boxSample(start, pointer) : boxSample(start, start);
    start = null;
    void editor.pickWhiteBalance(picked);
  }

  function onPointerCancel(e: PointerEvent): void {
    if (start?.id === e.pointerId) start = null;
  }

  function onPointerLeave(): void {
    if (start) return;
    pointer = null;
    probe.clear();
  }
</script>

<svelte:window onkeydown={onKeyDown} />

{#if active}
  <button
    type="button"
    data-testid="wb-picker-surface"
    class="absolute z-30 cursor-crosshair bg-transparent"
    style="left: {rect.x}px; top: {rect.y}px; width: {rect.w}px; height: {rect.h}px; touch-action: none;"
    aria-label="Sample neutral color"
    onpointerdown={onPointerDown}
    onpointermove={onPointerMove}
    onpointerup={onPointerUp}
    onpointercancel={onPointerCancel}
    onpointerleave={onPointerLeave}
  >
    {#if dragging && sample}
      <span
        data-testid="wb-picker-box"
        class="pointer-events-none absolute block border-2 border-primary bg-primary/15"
        style="left: {sample.u * 100}%; top: {sample.v * 100}%; width: {sample.w *
          100}%; height: {sample.h * 100}%;"
      ></span>
    {/if}
    {#if pointer}
      <span
        data-testid="wb-picker-readout"
        class="pointer-events-none absolute grid grid-cols-[auto_auto] gap-x-3 whitespace-nowrap rounded bg-black/75 px-2.5 py-1.5 text-left text-[11px] tabular-nums text-white shadow-lg"
        style="left: {pointer.u * 100}%; top: {pointer.v *
          100}%; transform: translate({cardShift});"
      >
        {#if reading?.wb}
          {@const wb = reading.wb}
          <span class="text-white/60">Temperature</span>
          <span class="text-right">{wb.mode === 'absolute' ? `${wb.kelvin} K` : wb.temp}</span>
          <span class="text-white/60">Tint</span>
          <span class="text-right">{wb.tint}</span>
        {:else if reading?.error}
          <span class="col-span-2">{reading.error}</span>
        {:else}
          <span class="col-span-2 text-white/60">Sampling…</span>
        {/if}
      </span>
    {/if}
  </button>
{/if}
