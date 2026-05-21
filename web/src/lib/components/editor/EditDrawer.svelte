<script lang="ts">
  import SliderRow from './SliderRow.svelte';
  import type { Edits } from '$lib/types/edits';
  import { NEUTRAL_EDITS, isIdentity } from '$lib/types/edits';

  let {
    edits = $bindable(),
    onLiveChange,
    onCommit,
    onReset,
    onExport,
    saving = false,
    exporting = false
  }: {
    edits: Edits;
    onLiveChange: () => void;
    onCommit: () => void;
    onReset: () => void;
    onExport: () => void;
    saving?: boolean;
    exporting?: boolean;
  } = $props();

  function rotateLeft(): void {
    edits.rotate = ((edits.rotate + 270) % 360) as 0 | 90 | 180 | 270;
    onCommit();
  }
  function rotateRight(): void {
    edits.rotate = ((edits.rotate + 90) % 360) as 0 | 90 | 180 | 270;
    onCommit();
  }
  function toggleFlipH(): void {
    edits.flip_h = !edits.flip_h;
    onCommit();
  }
  function toggleFlipV(): void {
    edits.flip_v = !edits.flip_v;
    onCommit();
  }
  function reset(): void {
    Object.assign(edits, NEUTRAL_EDITS);
    onReset();
  }
</script>

<aside class="flex flex-col gap-3 p-3 w-72 bg-base-200 overflow-y-auto">
  <div class="flex items-center justify-between">
    <h2 class="font-semibold">Edits</h2>
    <span class="text-xs opacity-60">
      {#if saving}saving…{:else if isIdentity(edits)}neutral{:else}edited{/if}
    </span>
  </div>

  <div class="flex flex-col gap-2">
    <SliderRow label="Exposure" bind:value={edits.exposure_ev} min={-5} max={5} step={0.05} onLive={onLiveChange} onCommit={onCommit} />
    <SliderRow label="Contrast" bind:value={edits.contrast} min={-100} max={100} step={1} onLive={onLiveChange} onCommit={onCommit} />
    <SliderRow label="Highlights" bind:value={edits.highlights} min={-100} max={100} step={1} onLive={onLiveChange} onCommit={onCommit} />
    <SliderRow label="Shadows" bind:value={edits.shadows} min={-100} max={100} step={1} onLive={onLiveChange} onCommit={onCommit} />
    <SliderRow label="Saturation" bind:value={edits.saturation} min={-100} max={100} step={1} onLive={onLiveChange} onCommit={onCommit} />
    <SliderRow label="WB Temp" bind:value={edits.wb_temp} min={-100} max={100} step={1} onLive={onLiveChange} onCommit={onCommit} />
    <SliderRow label="WB Tint" bind:value={edits.wb_tint} min={-100} max={100} step={1} onLive={onLiveChange} onCommit={onCommit} />
  </div>

  <div class="divider my-0"></div>
  <div class="flex flex-col gap-2">
    <span class="text-xs opacity-70">Transform</span>
    <div class="grid grid-cols-2 gap-2">
      <button class="btn btn-sm" onclick={rotateLeft}>⟲ 90°</button>
      <button class="btn btn-sm" onclick={rotateRight}>⟳ 90°</button>
      <button class="btn btn-sm" class:btn-active={edits.flip_h} onclick={toggleFlipH}>Flip H</button>
      <button class="btn btn-sm" class:btn-active={edits.flip_v} onclick={toggleFlipV}>Flip V</button>
    </div>
    <p class="text-xs opacity-50">rotate {edits.rotate}°</p>
  </div>

  <div class="divider my-0"></div>
  <div class="flex flex-col gap-2">
    <button class="btn btn-primary btn-sm" disabled={exporting} onclick={onExport}>
      {exporting ? 'Exporting…' : 'Export JPEG'}
    </button>
    <button class="btn btn-ghost btn-sm" onclick={reset}>Reset</button>
  </div>
</aside>
