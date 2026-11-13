<script lang="ts">
  import { editor } from '#lib/stores/editor.svelte.ts';
  import { WhiteBalanceProbe } from '#lib/stores/editor/whiteBalanceProbe.svelte.ts';
  import type { Edits } from '#lib/types/edits.ts';
  import type { PreviewSurface } from '#lib/utils/previewSurface.ts';
  import { imageRect } from '#lib/utils/imageRect.svelte.ts';
  import TouchMagnifier, { MAGNIFIER_LIFT, MAGNIFIER_SIZE } from './TouchMagnifier.svelte';
  import { PhotoPick } from '#lib/editor/photoPick.svelte.ts';

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
  const pick = new PhotoPick({
    busy: () => editor.wbBusy,
    track: (box) => {
      if (!editor.wbBusy) probe.probe(box);
    },
    pick: (box) => void editor.pickWhiteBalance(box)
  });

  const pointer = $derived(pick.pointer);
  const sample = $derived(pick.box);
  const reading = $derived(probe.reading);
  const cardShift = $derived(
    !pointer
      ? ''
      : pick.touching
        ? `-50%, calc(-100% - ${MAGNIFIER_LIFT + MAGNIFIER_SIZE + 8}px)`
        : `${pointer.u > 0.7 ? 'calc(-100% - 16px)' : '16px'}, ${pointer.v > 0.8 ? 'calc(-100% - 16px)' : '16px'}`
  );

  $effect(() => {
    if (!active) return;
    return () => {
      pick.reset();
      probe.clear();
    };
  });

  function onKeyDown(e: KeyboardEvent): void {
    if (e.key !== 'Escape' || !editor.wbPicking) return;
    e.preventDefault();
    editor.cancelWbPicker();
  }

  function onPointerLeave(): void {
    if (pick.leave()) probe.clear();
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
    {...pick.handlers}
    onpointerleave={onPointerLeave}
  >
    {#if pick.dragging && sample}
      <span
        data-testid="wb-picker-box"
        class="pointer-events-none absolute block border-2 border-image-accent bg-image-accent/15"
        style="left: {sample.u * 100}%; top: {sample.v * 100}%; width: {sample.w *
          100}%; height: {sample.h * 100}%;"
      ></span>
    {/if}
    {#if pick.touching && pointer && img}
      <TouchMagnifier source={img} u={pointer.u} v={pointer.v} width={rect.w} />
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
