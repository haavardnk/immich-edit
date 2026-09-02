<script lang="ts">
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import MaskedEditSlider from './MaskedEditSlider.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import { Button, Icon } from '@immich/ui';
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

<div class="mt-2 flex flex-col gap-1.5 border-t border-dark/10 pt-2">
  <div class="flex items-center gap-1 px-1">
    <div class="flex-1 min-w-0 text-[10px] uppercase tracking-wider text-dark/65 truncate">
      Adjustments · {layer?.name ?? ''}
    </div>
    <Button
      type="button"
      size="tiny"
      variant="ghost"
      color="secondary"
      class="size-6 shrink-0 p-0"
      title="Reset every adjustment on this mask"
      aria-label="Reset mask adjustments"
      disabled={isDefault}
      onclick={reset}
    >
      <Icon icon={mdiRestore} size="13px" aria-hidden="true" />
    </Button>
  </div>
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
  <div class="border-t border-dark/5"></div>
  <MaskedEditSlider
    {layerId}
    eKey="wb_temp"
    label="Temperature"
    min={-100}
    max={100}
    gradient="var(--gradient-temperature)"
  />
  <MaskedEditSlider
    {layerId}
    eKey="wb_tint"
    label="Tint"
    min={-100}
    max={100}
    gradient="var(--gradient-tint)"
  />
  <div class="border-t border-dark/5"></div>
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
  <div class="border-t border-dark/5"></div>
  <MaskedEditSlider {layerId} eKey="texture" label="Texture" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="clarity" label="Clarity" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="sharpen" label="Sharpening" min={-150} max={150} />
  <div class="border-t border-dark/5"></div>
  <MaskedEditSlider {layerId} eKey="saturation" label="Saturation" min={-100} max={100} />
  <MaskedEditSlider {layerId} eKey="vibrance" label="Vibrance" min={-100} max={100} />
</div>
