<script lang="ts">
  import Icon from '$lib/components/Icon.svelte';
  import SliderRow from '$lib/components/editor/controls/SliderRow.svelte';
  import { editor } from '$lib/stores/editor.svelte';
  import type { MaskComponent } from '$lib/types/edits';
  import { mdiEyedropperVariant } from '@mdi/js';

  let { layerId, component }: { layerId: string; component: MaskComponent } = $props();

  const luma = $derived(component.kind.kind === 'luma_range' ? component.kind : null);
  const color = $derived(component.kind.kind === 'color_range' ? component.kind : null);
  const softness = $derived(luma?.softness ?? color?.softness ?? 0.1);

  function commit(): void {
    void editor.commitMasks();
  }

  function updateLuma(field: 'min' | 'max' | 'softness', value: number): void {
    if (!luma) return;
    const next = { ...luma, [field]: value };
    if (field === 'min') next.min = Math.min(value, luma.max);
    if (field === 'max') next.max = Math.max(value, luma.min);
    editor.updateMaskComponentKind(layerId, component.id, next, true);
  }

  function updateColor(field: 'tolerance' | 'softness', value: number): void {
    if (!color) return;
    editor.updateMaskComponentKind(layerId, component.id, { ...color, [field]: value }, true);
  }

  function colorCss(rgb: [number, number, number]): string {
    const channels = rgb.map((value) => Math.round(Math.max(0, Math.min(1, value)) * 255));
    return `rgb(${channels[0]} ${channels[1]} ${channels[2]})`;
  }
</script>

{#if luma}
  <div class="mt-2 flex flex-col gap-2.5">
    <SliderRow
      label="Min"
      value={luma.min}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.25}
      onLive={(value: number) => updateLuma('min', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
    <SliderRow
      label="Max"
      value={luma.max}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.75}
      onLive={(value: number) => updateLuma('max', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
    <SliderRow
      label="Softness"
      value={softness}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.1}
      onLive={(value: number) => updateLuma('softness', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
  </div>
{/if}

{#if color}
  <div class="mt-2 flex flex-col gap-2.5">
    <div class="flex items-center justify-between px-1">
      <span class="text-[11px] text-immich-dark-fg/70">Sample</span>
      <div class="flex items-center gap-2">
        <span
          class="w-5 h-5 rounded-sm ring-1 ring-white/20"
          style="background-color: {colorCss(color.sample_rgb)}"
        ></span>
        <button
          type="button"
          class="inline-flex items-center justify-center w-6 h-6 rounded text-immich-dark-fg/60 hover:bg-white/10 hover:text-immich-dark-fg transition-colors {editor.colorPicker
            ? 'bg-white/10 text-immich-dark-primary'
            : ''}"
          title={editor.colorPicker ? 'Cancel eyedropper' : 'Pick color from image'}
          aria-label={editor.colorPicker ? 'Cancel eyedropper' : 'Pick color from image'}
          onclick={() => {
            if (editor.colorPicker) editor.cancelColorPicker();
            else editor.beginColorPicker(layerId, component.id);
          }}
        >
          <Icon path={mdiEyedropperVariant} size={14} />
        </button>
      </div>
    </div>
    <SliderRow
      label="Tolerance"
      value={color.tolerance}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.1}
      onLive={(value: number) => updateColor('tolerance', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
    <SliderRow
      label="Softness"
      value={softness}
      min={0}
      max={1}
      step={0.01}
      defaultValue={0.05}
      onLive={(value: number) => updateColor('softness', value)}
      onCommit={commit}
      format={(value: number) => value.toFixed(2)}
    />
  </div>
{/if}
