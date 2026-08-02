<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import MaskedEditSlider from './MaskedEditSlider.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { mdiRestore } from '@mdi/js';

  let { layerId }: { layerId: string } = $props();

  const layer = $derived(editor.edits.masks.find((l) => l.id === layerId) ?? null);
  const amountValue = $derived(layer?.amount ?? 1);
  const isDefault = $derived(
    amountValue === 1 && Object.values(layer?.edits ?? {}).every((v) => v === 0)
  );

  function onAmountLive(v: number): void {
    editor.setMaskLayerAmount(layerId, v);
  }

  function commit(): void {
    void editor.commitMasks();
  }

  function reset(): void {
    void editor.resetMaskLayerEdits(layerId);
  }
</script>

<div class="mt-3 border-t border-white/10 pt-3 flex flex-col gap-2.5">
  <div class="flex items-center gap-1 px-1">
    <div class="flex-1 min-w-0 text-[10px] uppercase tracking-wider text-immich-dark-fg/40 truncate">
      Adjustments · {layer?.name ?? ''}
    </div>
    <button
      type="button"
      class="shrink-0 text-immich-dark-fg/40 hover:text-immich-dark-fg transition-colors disabled:opacity-20 disabled:cursor-not-allowed"
      title="Reset every adjustment on this mask"
      aria-label="Reset mask adjustments"
      disabled={isDefault}
      onclick={reset}
    >
      <Icon path={mdiRestore} size={13} />
    </button>
  </div>
  <div class="px-1 text-[10px] text-immich-dark-fg/30">These sliders only affect the mask.</div>
  <SliderRow
    label="Amount"
    value={amountValue}
    min={0}
    max={1}
    step={0.01}
    defaultValue={1}
    onLive={onAmountLive}
    onCommit={commit}
    format={(v: number) => v.toFixed(2)}
  />
  <div class="border-t border-white/5"></div>
  <MaskedEditSlider
    {layerId}
    eKey="exposure_ev"
    label="Exposure"
    min={-5}
    max={5}
    step={0.05}
    format={(v: number) => v.toFixed(2)}
  />
  <MaskedEditSlider {layerId} eKey="brightness" label="Brightness" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="contrast" label="Contrast" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="highlights" label="Highlights" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="shadows" label="Shadows" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="whites" label="Whites" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="blacks" label="Blacks" min={-100} max={100} />
  <div class="border-t border-white/5"></div>
  <MaskedEditSlider {layerId} eKey="texture" label="Texture" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="clarity" label="Clarity" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="sharpen" label="Sharpening" min={-150} max={150} />
  <div class="border-t border-white/5"></div>
  <MaskedEditSlider {layerId} eKey="saturation" label="Saturation" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="vibrance" label="Vibrance" min={-100} max={100} />
  <div class="border-t border-white/5"></div>
  <MaskedEditSlider
    {layerId}
    eKey="wb_temp"
    label="Temperature"
    min={-100}
    max={100}
    gradient="linear-gradient(to right, #4a90d9, #b8a44c)"
  />
  <MaskedEditSlider
    {layerId}
    eKey="wb_tint"
    label="Tint"
    min={-100}
    max={100}
    gradient="linear-gradient(to right, #b8508a, #6ab04c)"
  />
</div>
