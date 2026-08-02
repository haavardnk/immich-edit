<script lang="ts">
  import MaskedEditSlider from './MaskedEditSlider.svelte';

  let { layerId }: { layerId: string } = $props();
</script>

<div class="mt-3 border-t border-white/10 pt-3 flex flex-col gap-2.5">
  <div class="text-[10px] uppercase tracking-wider text-immich-dark-fg/40 px-1">Adjustments</div>
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
